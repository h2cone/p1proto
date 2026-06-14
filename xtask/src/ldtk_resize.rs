use crate::cli::ResizeLdtkRoomsArgs;
use crate::paths::ProjectPaths;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const CRITICAL_ENTITY_IDENTIFIERS: &[&str] = &[
    "Checkpoint",
    "CollectibleStar",
    "CrumblingPlatform",
    "Ladder",
    "MovingPlatform",
    "PlainKey",
    "PlainLock",
    "Portal",
    "PressurePlate",
    "PushableCrate",
    "SwitchDoor",
    "WaterZone",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridSize {
    pub width: i64,
    pub height: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResizeOptions {
    pub target: GridSize,
    pub insert_x: Option<i64>,
    pub insert_y: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResizeProjectResult {
    pub project: Value,
    pub before_semantics: Value,
    pub after_semantics: Value,
    pub report: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InsertLine {
    axis: Axis,
    cell: i64,
    pixel: i64,
    score: i64,
    density: Option<i64>,
}

#[derive(Clone, Debug)]
struct IntGridResizeResult {
    csv: Vec<Value>,
    moved: i64,
    added: i64,
}

#[derive(Clone, Debug)]
struct TileResizeResult {
    tiles: Vec<Value>,
    moved: i64,
    added: i64,
}

#[derive(Clone, Debug)]
struct EntityResizeResult {
    moved: i64,
    held: i64,
    risks: Vec<Value>,
}

#[derive(Clone, Copy, Debug)]
struct EntityResizeParams {
    old_width: i64,
    old_height: i64,
    target_width: i64,
    target_height: i64,
    insert_x: i64,
    insert_y: i64,
    delta_width: i64,
    delta_height: i64,
    grid_size: i64,
    level_world_x: i64,
    level_world_y: i64,
}

pub fn execute(paths: &ProjectPaths, args: ResizeLdtkRoomsArgs) -> Result<()> {
    let path = paths.repo_root.join(&args.path);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let project: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    let result = resize_project_with_options(
        project,
        ResizeOptions {
            target: GridSize {
                width: args.width,
                height: args.height,
            },
            insert_x: args.insert_x,
            insert_y: args.insert_y,
        },
    )?;
    let project_json =
        serde_json::to_string_pretty(&result.project).context("failed to serialize LDtk JSON")?;

    if args.dry_run {
        if let Some(report_dir) = args.report_directory {
            let report_dir = paths.repo_root.join(report_dir);
            fs::create_dir_all(&report_dir)
                .with_context(|| format!("failed to create {}", report_dir.display()))?;
            write_json_report(
                &report_dir.join("before_semantics.json"),
                &result.before_semantics,
            )?;
            write_json_report(
                &report_dir.join("after_semantics.json"),
                &result.after_semantics,
            )?;
            write_json_report(&report_dir.join("dry_run_report.json"), &result.report)?;
            write_utf8_no_bom_file(&report_dir.join("dry_run_project.json"), &project_json)?;
        }
        println!(
            "Dry run only. Re-run without --dry-run to update {}",
            path.display()
        );
        print_report_summary(&result.report);
        return Ok(());
    }

    write_utf8_no_bom_file(&path, &project_json)
        .with_context(|| format!("failed to write {}", path.display()))?;
    println!("Updated {}", path.display());
    print_report_summary(&result.report);
    Ok(())
}

#[allow(dead_code)]
pub fn resize_project(project: Value, target: GridSize) -> Result<Value> {
    Ok(resize_project_with_options(
        project,
        ResizeOptions {
            target,
            insert_x: None,
            insert_y: None,
        },
    )?
    .project)
}

pub fn resize_project_with_options(
    mut project: Value,
    options: ResizeOptions,
) -> Result<ResizeProjectResult> {
    let before_semantics = new_semantics_snapshot(&project)?;
    let mut level_reports = Vec::new();

    set_property(&mut project, "worldGridWidth", json!(options.target.width))?;
    set_property(
        &mut project,
        "worldGridHeight",
        json!(options.target.height),
    )?;
    set_property(
        &mut project,
        "defaultLevelWidth",
        json!(options.target.width),
    )?;
    set_property(
        &mut project,
        "defaultLevelHeight",
        json!(options.target.height),
    )?;

    let levels = project
        .get_mut("levels")
        .and_then(Value::as_array_mut)
        .context("LDtk project is missing levels array")?;

    for level in levels {
        let old_width = required_i64(level, "pxWid")?;
        let old_height = required_i64(level, "pxHei")?;
        let grid_size = get_grid_size(level)?;
        if options.target.width % grid_size != 0 || options.target.height % grid_size != 0 {
            anyhow::bail!(
                "Target size {}x{} is not aligned to grid size {grid_size}.",
                options.target.width,
                options.target.height
            );
        }

        let (room_x, room_y) = room_coordinates(level)?;
        let new_world_x = room_x * options.target.width;
        let new_world_y = room_y * options.target.height;

        if options.target.width == old_width && options.target.height == old_height {
            resize_level_at_existing_size(
                level,
                GridSize {
                    width: old_width,
                    height: old_height,
                },
                grid_size,
                new_world_x,
                new_world_y,
                &mut level_reports,
            )?;
            continue;
        }

        if options.target.width <= old_width || options.target.height <= old_height {
            anyhow::bail!(
                "Target size must either match or expand both room axes. '{}' is currently {}x{}.",
                get_string(level, "identifier", ""),
                old_width,
                old_height
            );
        }

        let chosen_x = if let Some(insert_x) = options.insert_x {
            InsertLine {
                axis: Axis::X,
                cell: insert_x / grid_size,
                pixel: insert_x,
                score: 0,
                density: None,
            }
        } else {
            find_insert_line(level, Axis::X, options.target.width)?
        };
        let chosen_y = if let Some(insert_y) = options.insert_y {
            InsertLine {
                axis: Axis::Y,
                cell: insert_y / grid_size,
                pixel: insert_y,
                score: 0,
                density: None,
            }
        } else {
            find_insert_line(level, Axis::Y, options.target.height)?
        };

        if chosen_x.pixel % grid_size != 0 || chosen_y.pixel % grid_size != 0 {
            anyhow::bail!(
                "Insert lines for '{}' must be aligned to the {grid_size} px grid.",
                get_string(level, "identifier", "")
            );
        }

        resize_expanding_level(
            level,
            GridSize {
                width: old_width,
                height: old_height,
            },
            options.target,
            grid_size,
            chosen_x,
            chosen_y,
            new_world_x,
            new_world_y,
            &mut level_reports,
        )?;
    }

    let after_semantics = new_semantics_snapshot(&project)?;
    let validation_errors = test_resize_result(&project, &before_semantics, &after_semantics)?;
    let report = json!({
        "targetSize": [options.target.width, options.target.height],
        "levels": level_reports,
        "validationErrors": validation_errors,
    });

    if let Some(errors) = report["validationErrors"]
        .as_array()
        .filter(|errors| !errors.is_empty())
    {
        let joined = errors
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::bail!("Resize validation failed:\n{joined}");
    }

    Ok(ResizeProjectResult {
        project,
        before_semantics,
        after_semantics,
        report,
    })
}

fn resize_level_at_existing_size(
    level: &mut Value,
    size: GridSize,
    grid_size: i64,
    new_world_x: i64,
    new_world_y: i64,
    level_reports: &mut Vec<Value>,
) -> Result<()> {
    let old_width = required_i64(level, "pxWid")?;
    let old_height = required_i64(level, "pxHei")?;
    set_property(level, "worldX", json!(new_world_x))?;
    set_property(level, "worldY", json!(new_world_y))?;
    set_property(level, "pxWid", json!(size.width))?;
    set_property(level, "pxHei", json!(size.height))?;

    let mut held_entities = 0;
    let mut layer_reports = Vec::new();
    if let Some(layers) = level
        .get_mut("layerInstances")
        .and_then(Value::as_array_mut)
    {
        for layer in layers {
            let cell_width = size.width / grid_size;
            let cell_height = size.height / grid_size;
            set_property(layer, "__cWid", json!(cell_width))?;
            set_property(layer, "__cHei", json!(cell_height))?;

            let entity_count = if let Some(entities) = layer
                .get_mut("entityInstances")
                .and_then(Value::as_array_mut)
            {
                for entity in entities.iter_mut() {
                    let px = i64_pair(entity, "px");
                    if let Some([x, y]) = px {
                        set_property(entity, "__grid", json!([x / grid_size, y / grid_size]))?;
                        set_property(entity, "__worldX", json!(new_world_x + x))?;
                        set_property(entity, "__worldY", json!(new_world_y + y))?;
                    }
                }
                entities.len() as i64
            } else {
                0
            };

            held_entities += entity_count;
            layer_reports.push(json!({
                "identifier": get_string(layer, "__identifier", ""),
                "movedTiles": 0,
                "addedTiles": 0,
                "movedEntities": 0,
                "heldEntities": entity_count,
            }));
        }
    }

    level_reports.push(json!({
        "identifier": get_string(level, "identifier", ""),
        "oldSize": [old_width, old_height],
        "newSize": [size.width, size.height],
        "insert": Value::Null,
        "movedTiles": 0,
        "addedTiles": 0,
        "movedEntities": 0,
        "heldEntities": held_entities,
        "risks": [],
        "layers": layer_reports,
    }));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn resize_expanding_level(
    level: &mut Value,
    old_size: GridSize,
    target: GridSize,
    grid_size: i64,
    chosen_x: InsertLine,
    chosen_y: InsertLine,
    new_world_x: i64,
    new_world_y: i64,
    level_reports: &mut Vec<Value>,
) -> Result<()> {
    let old_cell_width = old_size.width / grid_size;
    let old_cell_height = old_size.height / grid_size;
    let new_cell_width = target.width / grid_size;
    let new_cell_height = target.height / grid_size;
    let delta_width = target.width - old_size.width;
    let delta_height = target.height - old_size.height;
    let delta_cells_x = delta_width / grid_size;
    let delta_cells_y = delta_height / grid_size;
    let insert_cell_x = chosen_x.pixel / grid_size;
    let insert_cell_y = chosen_y.pixel / grid_size;

    set_property(level, "worldX", json!(new_world_x))?;
    set_property(level, "worldY", json!(new_world_y))?;
    set_property(level, "pxWid", json!(target.width))?;
    set_property(level, "pxHei", json!(target.height))?;

    let mut level_moved_tiles = 0;
    let mut level_added_tiles = 0;
    let mut level_moved_entities = 0;
    let mut level_held_entities = 0;
    let mut level_risks = Vec::new();
    let mut layer_reports = Vec::new();

    if let Some(layers) = level
        .get_mut("layerInstances")
        .and_then(Value::as_array_mut)
    {
        for layer in layers {
            set_property(layer, "__cWid", json!(new_cell_width))?;
            set_property(layer, "__cHei", json!(new_cell_height))?;

            let mut layer_moved_tiles = 0;
            let mut layer_added_tiles = 0;
            let mut layer_moved_entities = 0;
            let mut layer_held_entities = 0;

            let int_grid = array_clone(layer, "intGridCsv");
            if !int_grid.is_empty() {
                let resized_grid = resize_int_grid_csv(
                    int_grid,
                    old_cell_width,
                    old_cell_height,
                    new_cell_width,
                    new_cell_height,
                    insert_cell_x,
                    insert_cell_y,
                    delta_cells_x,
                    delta_cells_y,
                )?;
                set_property(layer, "intGridCsv", Value::Array(resized_grid.csv))?;
                layer_moved_tiles += resized_grid.moved;
                layer_added_tiles += resized_grid.added;
            }

            for tile_set_name in ["gridTiles", "autoLayerTiles"] {
                let tiles = array_clone(layer, tile_set_name);
                if !tiles.is_empty() {
                    let resized_tiles = resize_tiles(
                        tiles,
                        old_cell_width,
                        old_cell_height,
                        new_cell_width,
                        grid_size,
                        insert_cell_x,
                        insert_cell_y,
                        delta_cells_x,
                        delta_cells_y,
                    )?;
                    set_property(layer, tile_set_name, Value::Array(resized_tiles.tiles))?;
                    layer_moved_tiles += resized_tiles.moved;
                    layer_added_tiles += resized_tiles.added;
                }
            }

            if let Some(entities) = layer
                .get_mut("entityInstances")
                .and_then(Value::as_array_mut)
                .filter(|entities| !entities.is_empty())
            {
                let entity_result = resize_entities(
                    entities,
                    EntityResizeParams {
                        old_width: old_size.width,
                        old_height: old_size.height,
                        target_width: target.width,
                        target_height: target.height,
                        insert_x: chosen_x.pixel,
                        insert_y: chosen_y.pixel,
                        delta_width,
                        delta_height,
                        grid_size,
                        level_world_x: new_world_x,
                        level_world_y: new_world_y,
                    },
                )?;
                layer_moved_entities += entity_result.moved;
                layer_held_entities += entity_result.held;
                level_risks.extend(entity_result.risks);
            }

            level_moved_tiles += layer_moved_tiles;
            level_added_tiles += layer_added_tiles;
            level_moved_entities += layer_moved_entities;
            level_held_entities += layer_held_entities;

            layer_reports.push(json!({
                "identifier": get_string(layer, "__identifier", ""),
                "movedTiles": layer_moved_tiles,
                "addedTiles": layer_added_tiles,
                "movedEntities": layer_moved_entities,
                "heldEntities": layer_held_entities,
            }));
        }
    }

    level_reports.push(json!({
        "identifier": get_string(level, "identifier", ""),
        "oldSize": [old_size.width, old_size.height],
        "newSize": [target.width, target.height],
        "insert": {
            "x": { "pixel": chosen_x.pixel, "cell": insert_cell_x, "density": chosen_x.density },
            "y": { "pixel": chosen_y.pixel, "cell": insert_cell_y, "density": chosen_y.density },
        },
        "movedTiles": level_moved_tiles,
        "addedTiles": level_added_tiles,
        "movedEntities": level_moved_entities,
        "heldEntities": level_held_entities,
        "risks": level_risks,
        "layers": layer_reports,
    }));
    Ok(())
}

fn get_grid_size(level: &Value) -> Result<i64> {
    for layer in array_clone(level, "layerInstances") {
        let grid_size = get_i64(&layer, "__gridSize", 0);
        if grid_size > 0 {
            return Ok(grid_size);
        }
    }

    anyhow::bail!(
        "Level '{}' does not contain a layer with a positive __gridSize.",
        get_string(level, "identifier", "")
    )
}

fn room_coordinates(level: &Value) -> Result<(i64, i64)> {
    let identifier = get_string(level, "identifier", "");
    let rest = identifier
        .strip_prefix("Room_")
        .with_context(|| format!("Level identifier '{identifier}' does not match Room_x_y."))?;
    let mut parts = rest.split('_');
    let x = parts
        .next()
        .context("missing room x")?
        .parse::<i64>()
        .with_context(|| format!("invalid room x in '{identifier}'"))?;
    let y = parts
        .next()
        .context("missing room y")?
        .parse::<i64>()
        .with_context(|| format!("invalid room y in '{identifier}'"))?;
    if parts.next().is_some() {
        anyhow::bail!("Level identifier '{identifier}' does not match Room_x_y.");
    }
    Ok((x, y))
}

fn move_axis_cell(cell: i64, insert_cell: i64, delta_cells: i64) -> i64 {
    if cell >= insert_cell {
        cell + delta_cells
    } else {
        cell
    }
}

fn move_axis_pixel(pixel: i64, insert_pixel: i64, delta_pixels: i64) -> i64 {
    if pixel >= insert_pixel {
        pixel + delta_pixels
    } else {
        pixel
    }
}

#[allow(dead_code)]
fn move_cell(cell: [i64; 2], offset: [i64; 2]) -> [i64; 2] {
    [cell[0] + offset[0], cell[1] + offset[1]]
}

#[allow(dead_code)]
fn move_pixel(pixel: [i64; 2], cell_offset: [i64; 2], grid_size: i64) -> [i64; 2] {
    [
        pixel[0] + cell_offset[0] * grid_size,
        pixel[1] + cell_offset[1] * grid_size,
    ]
}

fn cell_index(x: i64, y: i64, width: i64) -> usize {
    (y * width + x) as usize
}

fn tile_signature(tile: &Value) -> String {
    let src = tile
        .get("src")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(signature_value)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let tile_id = tile.get("t").map(signature_value).unwrap_or_default();
    let flip = tile.get("f").map(signature_value).unwrap_or_default();
    let alpha = tile.get("a").map(signature_value).unwrap_or_default();
    format!("{src}|{tile_id}|{flip}|{alpha}")
}

fn signature_value(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        text.to_owned()
    } else if let Some(number) = value.as_i64() {
        number.to_string()
    } else if value.is_null() {
        String::new()
    } else {
        value.to_string()
    }
}

fn set_tile_position(
    tile: &mut Value,
    cell_x: i64,
    cell_y: i64,
    grid_size: i64,
    layer_cell_width: i64,
) -> Result<()> {
    let pixel_x = cell_x * grid_size;
    let pixel_y = cell_y * grid_size;
    set_property(tile, "px", json!([pixel_x, pixel_y]))?;

    if let Some(data) = tile.get_mut("d").and_then(Value::as_array_mut) {
        if let Some(last) = data.last_mut() {
            *last = json!(cell_index(cell_x, cell_y, layer_cell_width) as i64);
        }
    }
    Ok(())
}

fn int_grid_value(csv: &[Value], x: i64, y: i64, width: i64) -> i64 {
    let index = cell_index(x, y, width);
    csv.get(index).and_then(Value::as_i64).unwrap_or(0)
}

#[allow(dead_code)]
fn resize_int_grid_layer(layer: &mut Value, new_width: i64, new_height: i64) -> Result<()> {
    let old_width = required_i64(layer, "__cWid")?;
    let old_height = required_i64(layer, "__cHei")?;
    let old = array_clone(layer, "intGridCsv");
    let resized = resize_int_grid_csv(
        old,
        old_width,
        old_height,
        new_width,
        new_height,
        old_width,
        old_height,
        new_width - old_width,
        new_height - old_height,
    )?;
    set_property(layer, "__cWid", json!(new_width))?;
    set_property(layer, "__cHei", json!(new_height))?;
    set_property(layer, "intGridCsv", Value::Array(resized.csv))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn resize_int_grid_csv(
    csv: Vec<Value>,
    old_cell_width: i64,
    old_cell_height: i64,
    new_cell_width: i64,
    new_cell_height: i64,
    insert_cell_x: i64,
    insert_cell_y: i64,
    delta_cells_x: i64,
    delta_cells_y: i64,
) -> Result<IntGridResizeResult> {
    let mut new_csv = vec![json!(0); (new_cell_width * new_cell_height) as usize];
    let mut moved = 0;
    let mut added = 0;

    for y in 0..old_cell_height {
        for x in 0..old_cell_width {
            let value = csv
                .get(cell_index(x, y, old_cell_width))
                .cloned()
                .unwrap_or_else(|| json!(0));
            let new_x = move_axis_cell(x, insert_cell_x, delta_cells_x);
            let new_y = move_axis_cell(y, insert_cell_y, delta_cells_y);
            new_csv[cell_index(new_x, new_y, new_cell_width)] = value.clone();

            if value.as_i64().unwrap_or(0) != 0 && (new_x != x || new_y != y) {
                moved += 1;
            }
        }
    }

    if delta_cells_x > 0 && insert_cell_x > 0 && insert_cell_x < old_cell_width {
        for y in 0..old_cell_height {
            let left = int_grid_value(&csv, insert_cell_x - 1, y, old_cell_width);
            let right = int_grid_value(&csv, insert_cell_x, y, old_cell_width);
            if left != 0 && left == right {
                let new_y = move_axis_cell(y, insert_cell_y, delta_cells_y);
                for fill_x in insert_cell_x..(insert_cell_x + delta_cells_x) {
                    let index = cell_index(fill_x, new_y, new_cell_width);
                    if new_csv[index].as_i64().unwrap_or(0) == 0 {
                        new_csv[index] = json!(left);
                        added += 1;
                    }
                }
            }
        }
    }

    if delta_cells_y > 0 && insert_cell_y > 0 && insert_cell_y < old_cell_height {
        for x in 0..old_cell_width {
            let top = int_grid_value(&csv, x, insert_cell_y - 1, old_cell_width);
            let bottom = int_grid_value(&csv, x, insert_cell_y, old_cell_width);
            if top != 0 && top == bottom {
                let new_x = move_axis_cell(x, insert_cell_x, delta_cells_x);
                for fill_y in insert_cell_y..(insert_cell_y + delta_cells_y) {
                    let index = cell_index(new_x, fill_y, new_cell_width);
                    if new_csv[index].as_i64().unwrap_or(0) == 0 {
                        new_csv[index] = json!(top);
                        added += 1;
                    }
                }
            }
        }
    }

    Ok(IntGridResizeResult {
        csv: new_csv,
        moved,
        added,
    })
}

#[derive(Clone, Debug)]
struct TileRecord {
    tile: Value,
    x: i64,
    y: i64,
    signature: String,
}

#[allow(clippy::too_many_arguments)]
fn resize_tiles(
    tiles: Vec<Value>,
    old_cell_width: i64,
    old_cell_height: i64,
    new_cell_width: i64,
    grid_size: i64,
    insert_cell_x: i64,
    insert_cell_y: i64,
    delta_cells_x: i64,
    delta_cells_y: i64,
) -> Result<TileResizeResult> {
    let mut records = Vec::new();
    let mut by_old_coord = HashMap::new();

    for tile in tiles {
        let Some([pixel_x, pixel_y]) = i64_pair(&tile, "px") else {
            continue;
        };
        let x = pixel_x / grid_size;
        let y = pixel_y / grid_size;
        let record = TileRecord {
            signature: tile_signature(&tile),
            tile,
            x,
            y,
        };
        by_old_coord.insert((x, y), records.len());
        records.push(record);
    }

    let mut output = Vec::new();
    let mut occupied_new = HashSet::new();
    let mut moved = 0;

    for record in &mut records {
        let new_x = move_axis_cell(record.x, insert_cell_x, delta_cells_x);
        let new_y = move_axis_cell(record.y, insert_cell_y, delta_cells_y);
        if new_x != record.x || new_y != record.y {
            moved += 1;
        }

        set_tile_position(&mut record.tile, new_x, new_y, grid_size, new_cell_width)?;
        output.push(record.tile.clone());
        occupied_new.insert((new_x, new_y));
    }

    let mut added = 0;
    if delta_cells_x > 0 && insert_cell_x > 0 && insert_cell_x < old_cell_width {
        for y in 0..old_cell_height {
            let left = by_old_coord.get(&(insert_cell_x - 1, y)).copied();
            let right = by_old_coord.get(&(insert_cell_x, y)).copied();
            if let (Some(left), Some(right)) = (left, right) {
                if records[left].signature == records[right].signature {
                    let new_y = move_axis_cell(y, insert_cell_y, delta_cells_y);
                    for fill_x in insert_cell_x..(insert_cell_x + delta_cells_x) {
                        if occupied_new.insert((fill_x, new_y)) {
                            let mut clone = records[left].tile.clone();
                            set_tile_position(
                                &mut clone,
                                fill_x,
                                new_y,
                                grid_size,
                                new_cell_width,
                            )?;
                            output.push(clone);
                            added += 1;
                        }
                    }
                }
            }
        }
    }

    if delta_cells_y > 0 && insert_cell_y > 0 && insert_cell_y < old_cell_height {
        for x in 0..old_cell_width {
            let top = by_old_coord.get(&(x, insert_cell_y - 1)).copied();
            let bottom = by_old_coord.get(&(x, insert_cell_y)).copied();
            if let (Some(top), Some(bottom)) = (top, bottom) {
                if records[top].signature == records[bottom].signature {
                    let new_x = move_axis_cell(x, insert_cell_x, delta_cells_x);
                    for fill_y in insert_cell_y..(insert_cell_y + delta_cells_y) {
                        if occupied_new.insert((new_x, fill_y)) {
                            let mut clone = records[top].tile.clone();
                            set_tile_position(
                                &mut clone,
                                new_x,
                                fill_y,
                                grid_size,
                                new_cell_width,
                            )?;
                            output.push(clone);
                            added += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(TileResizeResult {
        tiles: output,
        moved,
        added,
    })
}

fn coordinate_field_risks(entity: &Value) -> Vec<Value> {
    let mut risks = Vec::new();
    for field in array_clone(entity, "fieldInstances") {
        let identifier = get_string(&field, "__identifier", "");
        let field_type = get_string(&field, "__type", "");
        let value = field.get("__value").cloned().unwrap_or(Value::Null);
        let looks_like_point = field_type.contains("Point")
            || value.as_object().is_some_and(|object| {
                (object.contains_key("cx") && object.contains_key("cy"))
                    || (object.contains_key("x") && object.contains_key("y"))
            });

        if looks_like_point {
            risks.push(json!({
                "entity": get_string(entity, "__identifier", ""),
                "iid": get_string(entity, "iid", ""),
                "field": identifier,
                "type": field_type,
                "reason": "coordinate-like field is preserved for manual review",
            }));
        }
    }
    risks
}

fn resize_entities(
    entities: &mut [Value],
    params: EntityResizeParams,
) -> Result<EntityResizeResult> {
    let mut moved = 0;
    let mut held = 0;
    let mut risks = Vec::new();

    for entity in entities {
        let Some([old_x, old_y]) = i64_pair(entity, "px") else {
            continue;
        };
        let width = get_i64(entity, "width", params.grid_size);
        let height = get_i64(entity, "height", params.grid_size);
        let identifier = get_string(entity, "__identifier", "");
        let iid = get_string(entity, "iid", "");

        if (old_x < params.insert_x && old_x + width > params.insert_x)
            || (old_y < params.insert_y && old_y + height > params.insert_y)
        {
            risks.push(json!({
                "entity": identifier,
                "iid": iid,
                "position": [old_x, old_y],
                "reason": "entity bounds cross an insertion line; local size was preserved",
            }));
        }

        risks.extend(coordinate_field_risks(entity));

        let touches_right = width > 0 && old_x + width >= params.old_width;
        let touches_bottom = height > 0 && old_y + height >= params.old_height;

        let new_x = if touches_right {
            params.target_width - width
        } else {
            move_axis_pixel(old_x, params.insert_x, params.delta_width)
        };
        let new_y = if touches_bottom {
            params.target_height - height
        } else {
            move_axis_pixel(old_y, params.insert_y, params.delta_height)
        };

        if new_x != old_x || new_y != old_y {
            moved += 1;
        } else {
            held += 1;
        }

        set_property(entity, "px", json!([new_x, new_y]))?;
        set_property(
            entity,
            "__grid",
            json!([new_x / params.grid_size, new_y / params.grid_size]),
        )?;
        set_property(entity, "__worldX", json!(params.level_world_x + new_x))?;
        set_property(entity, "__worldY", json!(params.level_world_y + new_y))?;
    }

    Ok(EntityResizeResult { moved, held, risks })
}

pub fn find_insert_line(level: &Value, axis: Axis, target_pixels: i64) -> Result<InsertLine> {
    let grid_size = get_grid_size(level)?;
    let old_pixels = match axis {
        Axis::X => required_i64(level, "pxWid")?,
        Axis::Y => required_i64(level, "pxHei")?,
    };
    if target_pixels <= old_pixels {
        anyhow::bail!("Target {axis:?} size must be larger than the current size.");
    }
    if (target_pixels - old_pixels) % grid_size != 0 {
        anyhow::bail!("Target {axis:?} size must stay aligned to the {grid_size} px grid.");
    }

    let old_cells = old_pixels / grid_size;
    let center_cell = old_cells / 2;
    let mut best: Option<InsertLine> = None;

    for cell in 1..old_cells {
        let pixel = cell * grid_size;
        let mut blocked = false;
        let mut density = 0;
        let mut entity_origin_penalty = 0;

        for layer in array_clone(level, "layerInstances") {
            let layer_grid = get_i64(&layer, "__gridSize", grid_size);
            let layer_width = get_i64(&layer, "__cWid", 0);
            let layer_height = get_i64(&layer, "__cHei", 0);

            for tile_set_name in ["gridTiles", "autoLayerTiles"] {
                for tile in array_clone(&layer, tile_set_name) {
                    let Some([pixel_x, pixel_y]) = i64_pair(&tile, "px") else {
                        continue;
                    };
                    let tile_cell = match axis {
                        Axis::X => pixel_x / layer_grid,
                        Axis::Y => pixel_y / layer_grid,
                    };
                    if tile_cell == cell {
                        density += 1;
                    }
                }
            }

            let int_grid = array_clone(&layer, "intGridCsv");
            if !int_grid.is_empty() && layer_width > 0 && layer_height > 0 {
                match axis {
                    Axis::X if cell < layer_width => {
                        for y in 0..layer_height {
                            if int_grid_value(&int_grid, cell, y, layer_width) != 0 {
                                density += 1;
                            }
                        }
                    }
                    Axis::Y if cell < layer_height => {
                        for x in 0..layer_width {
                            if int_grid_value(&int_grid, x, cell, layer_width) != 0 {
                                density += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }

            for entity in array_clone(&layer, "entityInstances") {
                let identifier = get_string(&entity, "__identifier", "");
                let Some([entity_x, entity_y]) = i64_pair(&entity, "px") else {
                    continue;
                };
                let width = get_i64(&entity, "width", grid_size);
                let height = get_i64(&entity, "height", grid_size);
                let (start, extent) = match axis {
                    Axis::X => (entity_x, width),
                    Axis::Y => (entity_y, height),
                };

                if CRITICAL_ENTITY_IDENTIFIERS.contains(&identifier.as_str())
                    && ((pixel > start && pixel < start + extent) || pixel == start)
                {
                    blocked = true;
                    break;
                }

                if pixel == start {
                    entity_origin_penalty += 20;
                }
            }

            if blocked {
                break;
            }
        }

        if blocked {
            continue;
        }

        let score = density * 100 + entity_origin_penalty + (cell - center_cell).abs();
        if best.is_none_or(|current| score < current.score) {
            best = Some(InsertLine {
                axis,
                cell,
                pixel,
                score,
                density: Some(density),
            });
        }
    }

    best.with_context(|| {
        format!(
            "No safe {axis:?} insertion line found for level '{}'.",
            get_string(level, "identifier", "")
        )
    })
}

fn new_semantics_snapshot(project: &Value) -> Result<Value> {
    let mut levels_snapshot = Vec::new();
    for level in array_clone(project, "levels") {
        let mut entities_snapshot = Vec::new();
        let mut layers_snapshot = Vec::new();

        for layer in array_clone(&level, "layerInstances") {
            let entity_instances = array_clone(&layer, "entityInstances");
            let int_grid_values = array_clone(&layer, "intGridCsv")
                .into_iter()
                .filter(|value| value.as_i64().unwrap_or(0) != 0)
                .count();
            let grid_tiles = array_clone(&layer, "gridTiles");
            let auto_layer_tiles = array_clone(&layer, "autoLayerTiles");

            for entity in &entity_instances {
                let mut field_snapshots = Vec::new();
                for field in array_clone(entity, "fieldInstances") {
                    field_snapshots.push(json!({
                        "identifier": get_string(&field, "__identifier", ""),
                        "type": get_string(&field, "__type", ""),
                        "value": field.get("__value").cloned().unwrap_or(Value::Null),
                    }));
                }

                entities_snapshot.push(json!({
                    "identifier": get_string(entity, "__identifier", ""),
                    "iid": get_string(entity, "iid", ""),
                    "px": array_clone(entity, "px"),
                    "grid": array_clone(entity, "__grid"),
                    "world": [
                        entity.get("__worldX").cloned().unwrap_or(Value::Null),
                        entity.get("__worldY").cloned().unwrap_or(Value::Null),
                    ],
                    "width": entity.get("width").cloned().unwrap_or(Value::Null),
                    "height": entity.get("height").cloned().unwrap_or(Value::Null),
                    "fields": field_snapshots,
                }));
            }

            layers_snapshot.push(json!({
                "identifier": get_string(&layer, "__identifier", ""),
                "type": get_string(&layer, "__type", ""),
                "cellWidth": get_i64(&layer, "__cWid", 0),
                "cellHeight": get_i64(&layer, "__cHei", 0),
                "intGridValues": int_grid_values,
                "gridTiles": grid_tiles.len(),
                "autoLayerTiles": auto_layer_tiles.len(),
                "entities": entity_instances.len(),
            }));
        }

        levels_snapshot.push(json!({
            "identifier": get_string(&level, "identifier", ""),
            "iid": get_string(&level, "iid", ""),
            "world": [required_i64(&level, "worldX")?, required_i64(&level, "worldY")?],
            "size": [required_i64(&level, "pxWid")?, required_i64(&level, "pxHei")?],
            "layers": layers_snapshot,
            "entities": entities_snapshot,
        }));
    }

    Ok(json!({
        "worldGridWidth": get_i64(project, "worldGridWidth", 0),
        "worldGridHeight": get_i64(project, "worldGridHeight", 0),
        "defaultLevelWidth": get_i64(project, "defaultLevelWidth", 0),
        "defaultLevelHeight": get_i64(project, "defaultLevelHeight", 0),
        "levels": levels_snapshot,
    }))
}

fn test_resize_result(
    project: &Value,
    before_semantics: &Value,
    after_semantics: &Value,
) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    for level in array_clone(project, "levels") {
        let level_width = required_i64(&level, "pxWid")?;
        let level_height = required_i64(&level, "pxHei")?;
        for layer in array_clone(&level, "layerInstances") {
            let grid_size = get_i64(&layer, "__gridSize", 0);
            let cell_width = get_i64(&layer, "__cWid", 0);
            let cell_height = get_i64(&layer, "__cHei", 0);
            if grid_size > 0 {
                if cell_width * grid_size != level_width {
                    errors.push(format!(
                        "{}/{}: __cWid * gridSize does not match pxWid",
                        get_string(&level, "identifier", ""),
                        get_string(&layer, "__identifier", "")
                    ));
                }
                if cell_height * grid_size != level_height {
                    errors.push(format!(
                        "{}/{}: __cHei * gridSize does not match pxHei",
                        get_string(&level, "identifier", ""),
                        get_string(&layer, "__identifier", "")
                    ));
                }
            }

            let int_grid = array_clone(&layer, "intGridCsv");
            if !int_grid.is_empty() && int_grid.len() as i64 != cell_width * cell_height {
                errors.push(format!(
                    "{}/{}: intGridCsv length mismatch",
                    get_string(&level, "identifier", ""),
                    get_string(&layer, "__identifier", "")
                ));
            }

            for tile_set_name in ["gridTiles", "autoLayerTiles"] {
                for tile in array_clone(&layer, tile_set_name) {
                    if let Some([x, y]) = i64_pair(&tile, "px") {
                        if x < 0 || y < 0 || x >= level_width || y >= level_height {
                            errors.push(format!(
                                "{}/{}: tile outside resized room",
                                get_string(&level, "identifier", ""),
                                get_string(&layer, "__identifier", "")
                            ));
                        }
                    }
                }
            }

            for entity in array_clone(&layer, "entityInstances") {
                if let Some([x, y]) = i64_pair(&entity, "px") {
                    if x < 0 || y < 0 || x >= level_width || y >= level_height {
                        errors.push(format!(
                            "{}/{}: entity outside resized room",
                            get_string(&level, "identifier", ""),
                            get_string(&layer, "__identifier", "")
                        ));
                    }
                }
            }
        }
    }

    let before_entities = entity_identity_set(before_semantics);
    let after_entities = entity_identity_set(after_semantics);
    for key in &before_entities {
        if !after_entities.contains(key) {
            errors.push(format!("Entity identity changed or disappeared: {key}"));
        }
    }
    for key in &after_entities {
        if !before_entities.contains(key) {
            errors.push(format!("New unexpected entity identity appeared: {key}"));
        }
    }

    Ok(errors)
}

fn entity_identity_set(snapshot: &Value) -> HashSet<String> {
    let mut entities = HashSet::new();
    for level in array_clone(snapshot, "levels") {
        for entity in array_clone(&level, "entities") {
            entities.insert(format!(
                "{}|{}|{}",
                get_string(&level, "identifier", ""),
                get_string(&entity, "iid", ""),
                get_string(&entity, "identifier", "")
            ));
        }
    }
    entities
}

fn write_json_report(path: &Path, value: &Value) -> Result<()> {
    let content = serde_json::to_string_pretty(value).context("failed to serialize report JSON")?;
    write_utf8_no_bom_file(path, &content)
}

fn write_utf8_no_bom_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))
}

fn print_report_summary(report: &Value) {
    for level in array_clone(report, "levels") {
        let insert_summary = if level.get("insert").is_some_and(Value::is_null) {
            "already target size".to_owned()
        } else {
            let x = level["insert"]["x"]["pixel"].as_i64().unwrap_or_default();
            let y = level["insert"]["y"]["pixel"].as_i64().unwrap_or_default();
            format!("insert x={x}px y={y}px")
        };
        let old = array_clone(&level, "oldSize");
        let new = array_clone(&level, "newSize");
        println!(
            "{}: {}x{} -> {}x{}; {}; moved tiles={}; added tiles={}; moved entities={}; risks={}",
            get_string(&level, "identifier", ""),
            old.first().and_then(Value::as_i64).unwrap_or_default(),
            old.get(1).and_then(Value::as_i64).unwrap_or_default(),
            new.first().and_then(Value::as_i64).unwrap_or_default(),
            new.get(1).and_then(Value::as_i64).unwrap_or_default(),
            insert_summary,
            get_i64(&level, "movedTiles", 0),
            get_i64(&level, "addedTiles", 0),
            get_i64(&level, "movedEntities", 0),
            array_clone(&level, "risks").len(),
        );
    }
}

fn set_property(object: &mut Value, name: &str, value: Value) -> Result<()> {
    let map = object.as_object_mut().context("expected JSON object")?;
    map.insert(name.to_owned(), value);
    Ok(())
}

fn required_i64(object: &Value, name: &str) -> Result<i64> {
    object
        .get(name)
        .and_then(Value::as_i64)
        .with_context(|| format!("missing integer field '{name}'"))
}

fn get_i64(object: &Value, name: &str, default: i64) -> i64 {
    object.get(name).and_then(Value::as_i64).unwrap_or(default)
}

fn get_string(object: &Value, name: &str, default: &str) -> String {
    object
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

fn array_clone(object: &Value, name: &str) -> Vec<Value> {
    object
        .get(name)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn i64_pair(object: &Value, name: &str) -> Option<[i64; 2]> {
    let values = object.get(name)?.as_array()?;
    Some([values.first()?.as_i64()?, values.get(1)?.as_i64()?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn test_project() -> Value {
        serde_json::from_str(
            r#"
{
  "worldGridWidth": 32,
  "worldGridHeight": 24,
  "defaultLevelWidth": 32,
  "defaultLevelHeight": 24,
  "levels": [
    {
      "identifier": "Room_1_2",
      "iid": "level-1",
      "worldX": 32,
      "worldY": 48,
      "pxWid": 32,
      "pxHei": 24,
      "fieldInstances": [],
      "layerInstances": [
        {
          "__identifier": "Hazards",
          "__type": "IntGrid",
          "__cWid": 4,
          "__cHei": 3,
          "__gridSize": 8,
          "intGridCsv": [0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0],
          "autoLayerTiles": [],
          "gridTiles": [],
          "entityInstances": []
        },
        {
          "__identifier": "Tiles",
          "__type": "Tiles",
          "__cWid": 4,
          "__cHei": 3,
          "__gridSize": 8,
          "intGridCsv": [],
          "autoLayerTiles": [],
          "gridTiles": [
            { "px": [8, 16], "src": [0, 0], "f": 0, "t": 1, "d": [9], "a": 1 },
            { "px": [16, 16], "src": [0, 0], "f": 0, "t": 1, "d": [10], "a": 1 },
            { "px": [24, 16], "src": [0, 0], "f": 0, "t": 1, "d": [11], "a": 1 }
          ],
          "entityInstances": []
        },
        {
          "__identifier": "Entities",
          "__type": "Entities",
          "__cWid": 4,
          "__cHei": 3,
          "__gridSize": 8,
          "intGridCsv": [],
          "autoLayerTiles": [],
          "gridTiles": [],
          "entityInstances": [
            {
              "__identifier": "PlainKey",
              "iid": "key-1",
              "px": [24, 8],
              "__grid": [3, 1],
              "__worldX": 56,
              "__worldY": 56,
              "width": 8,
              "height": 8,
              "fieldInstances": []
            },
            {
              "__identifier": "Portal",
              "iid": "portal-1",
              "px": [0, 0],
              "__grid": [0, 0],
              "__worldX": 32,
              "__worldY": 48,
              "width": 8,
              "height": 8,
              "fieldInstances": [
                { "__identifier": "dest_x", "__type": "Int", "__value": 0 },
                { "__identifier": "dest_y", "__type": "Int", "__value": 1 }
              ]
            }
          ]
        }
      ]
    }
  ]
}
"#,
        )
        .unwrap()
    }

    #[test]
    fn idempotent_when_level_already_matches_target_size() {
        let project = test_project();
        let resized = resize_project(
            project.clone(),
            GridSize {
                width: 32,
                height: 24,
            },
        )
        .unwrap();
        assert_eq!(resized, project);
    }

    #[test]
    fn resizes_int_grid_csv_with_zero_fill() {
        let mut layer = json!({
            "__type": "IntGrid",
            "__cWid": 2,
            "__cHei": 2,
            "intGridCsv": [1, 2, 3, 4]
        });

        resize_int_grid_layer(&mut layer, 3, 2).unwrap();

        assert_eq!(layer["__cWid"], 3);
        assert_eq!(layer["__cHei"], 2);
        assert_eq!(layer["intGridCsv"], json!([1, 2, 0, 3, 4, 0]));
    }

    #[test]
    fn moves_grid_cell_coordinates_by_offset() {
        let moved = move_cell([2, 3], [4, -1]);
        assert_eq!(moved, [6, 2]);
    }

    #[test]
    fn moves_pixel_coordinates_by_grid_offset() {
        let moved = move_pixel([16, 24], [2, 1], 8);
        assert_eq!(moved, [32, 32]);
    }

    #[test]
    fn resize_project_moves_room_data() {
        let project = test_project();
        let result = resize_project_with_options(
            project,
            ResizeOptions {
                target: GridSize {
                    width: 48,
                    height: 32,
                },
                insert_x: Some(16),
                insert_y: Some(16),
            },
        )
        .unwrap();
        let project = result.project;

        assert_eq!(project["worldGridWidth"], 48);
        assert_eq!(project["worldGridHeight"], 32);
        assert_eq!(project["defaultLevelWidth"], 48);
        assert_eq!(project["defaultLevelHeight"], 32);

        let level = &project["levels"][0];
        assert_eq!(
            json!([
                level["worldX"],
                level["worldY"],
                level["pxWid"],
                level["pxHei"]
            ]),
            json!([48, 64, 48, 32])
        );

        let hazards = layer(level, "Hazards");
        assert_eq!(json!([hazards["__cWid"], hazards["__cHei"]]), json!([6, 4]));
        assert_eq!(hazards["intGridCsv"][1], 7);
        assert_eq!(hazards["intGridCsv"][22], 9);

        let entities = layer(level, "Entities");
        let key = entity(entities, "key-1");
        let portal = entity(entities, "portal-1");
        assert_eq!(
            json!([
                key["px"][0],
                key["px"][1],
                key["__grid"][0],
                key["__grid"][1],
                key["__worldX"],
                key["__worldY"]
            ]),
            json!([40, 8, 5, 1, 88, 72])
        );
        assert_eq!(
            json!([
                portal["px"][0],
                portal["px"][1],
                portal["__grid"][0],
                portal["__grid"][1],
                portal["__worldX"],
                portal["__worldY"]
            ]),
            json!([0, 0, 0, 0, 48, 64])
        );
        assert_eq!(
            portal["fieldInstances"]
                .as_array()
                .unwrap()
                .iter()
                .map(|field| field["__value"].clone())
                .collect::<Vec<_>>(),
            vec![json!(0), json!(1)]
        );

        assert_eq!(result.report["levels"][0]["movedEntities"], 1);
        assert_eq!(result.report["levels"][0]["heldEntities"], 1);
    }

    #[test]
    fn resize_project_extends_continuous_tile_segments() {
        let result = resize_project_with_options(
            test_project(),
            ResizeOptions {
                target: GridSize {
                    width: 48,
                    height: 32,
                },
                insert_x: Some(16),
                insert_y: Some(16),
            },
        )
        .unwrap();
        let tiles = layer(&result.project["levels"][0], "Tiles")["gridTiles"]
            .as_array()
            .unwrap();
        let mut coords = tiles
            .iter()
            .map(|tile| {
                (
                    tile["px"][0].as_i64().unwrap(),
                    tile["px"][1].as_i64().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        coords.sort_by_key(|(x, y)| (*y, *x));

        assert_eq!(
            coords,
            vec![(8, 24), (16, 24), (24, 24), (32, 24), (40, 24)]
        );
    }

    #[test]
    fn find_safe_insert_line_avoids_critical_entities() {
        let project = test_project();
        let level = &project["levels"][0];
        let insert = find_insert_line(level, Axis::X, 48).unwrap();

        assert_ne!(insert.pixel, 24);
        assert_eq!(insert.pixel % 8, 0);
    }

    #[test]
    fn resize_project_is_idempotent_at_target_size() {
        let first = resize_project_with_options(
            test_project(),
            ResizeOptions {
                target: GridSize {
                    width: 48,
                    height: 32,
                },
                insert_x: Some(16),
                insert_y: Some(16),
            },
        )
        .unwrap();

        let second = resize_project_with_options(
            first.project,
            ResizeOptions {
                target: GridSize {
                    width: 48,
                    height: 32,
                },
                insert_x: None,
                insert_y: None,
            },
        )
        .unwrap();

        assert_eq!(second.report["levels"][0]["oldSize"], json!([48, 32]));
        assert_eq!(second.report["levels"][0]["newSize"], json!([48, 32]));
        assert_eq!(second.report["levels"][0]["movedTiles"], 0);
        assert_eq!(second.report["levels"][0]["movedEntities"], 0);
    }

    #[test]
    fn write_utf8_no_bom_file_omits_bom() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        write_utf8_no_bom_file(temp.path(), "{\"ok\":true}").unwrap();
        let bytes = std::fs::read(temp.path()).unwrap();

        assert!(bytes.len() >= 3);
        assert_ne!(&bytes[0..3], [0xEF, 0xBB, 0xBF]);
    }

    fn layer<'a>(level: &'a Value, identifier: &str) -> &'a Value {
        level["layerInstances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|layer| layer["__identifier"] == identifier)
            .unwrap()
    }

    fn entity<'a>(layer: &'a Value, iid: &str) -> &'a Value {
        layer["entityInstances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entity| entity["iid"] == iid)
            .unwrap()
    }
}
