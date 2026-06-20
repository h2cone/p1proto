@tool
extends RefCounted

const SOLID_PHYSICS_LAYER := 0
const HAZARD_PHYSICS_LAYER := 1
const TILE_COLLISION_LAYER := 1 << (3 - 1)
const HAZARD_COLLISION_LAYER := 1 << (12 - 1)
const SOLID_TILE_COORDS := [Vector2i(6, 2)]
const HAZARD_TILE_COORDS := [Vector2i(8, 0)]


func post_import(tilesets: Dictionary) -> Dictionary:
	for tileset: TileSet in tilesets.values():
		_configure_tileset(tileset)
	return tilesets


func _configure_tileset(tileset: TileSet) -> void:
	_ensure_physics_layers(tileset)
	tileset.set_physics_layer_collision_layer(SOLID_PHYSICS_LAYER, TILE_COLLISION_LAYER)
	tileset.set_physics_layer_collision_layer(HAZARD_PHYSICS_LAYER, HAZARD_COLLISION_LAYER)

	for source_index in range(tileset.get_source_count()):
		var source_id := tileset.get_source_id(source_index)
		var source := tileset.get_source(source_id)
		if not source is TileSetAtlasSource:
			continue

		for coords: Vector2i in SOLID_TILE_COORDS:
			_configure_tile(tileset, source, coords, SOLID_PHYSICS_LAYER)
		for coords: Vector2i in HAZARD_TILE_COORDS:
			_configure_tile(tileset, source, coords, HAZARD_PHYSICS_LAYER)


func _ensure_physics_layers(tileset: TileSet) -> void:
	while tileset.get_physics_layers_count() <= HAZARD_PHYSICS_LAYER:
		tileset.add_physics_layer()


func _configure_tile(
		tileset: TileSet,
		source: TileSetAtlasSource,
		coords: Vector2i,
		target_layer: int
) -> void:
	if source.get_tile_at_coords(coords) == Vector2i(-1, -1):
		return

	var tile_data := source.get_tile_data(coords, 0)
	if tile_data == null:
		return

	var points := _collect_collision_points(tile_data, target_layer)
	if points.is_empty():
		points = _collect_collision_points(tile_data, SOLID_PHYSICS_LAYER)
	if points.is_empty():
		points = [_full_tile_polygon(tileset)]

	tile_data.set_collision_polygons_count(SOLID_PHYSICS_LAYER, 0)
	tile_data.set_collision_polygons_count(HAZARD_PHYSICS_LAYER, 0)
	_write_collision_points(tile_data, target_layer, points)


func _collect_collision_points(tile_data: TileData, layer_id: int) -> Array[PackedVector2Array]:
	var points: Array[PackedVector2Array] = []
	for polygon_index in range(tile_data.get_collision_polygons_count(layer_id)):
		points.append(tile_data.get_collision_polygon_points(layer_id, polygon_index))
	return points


func _write_collision_points(
		tile_data: TileData,
		layer_id: int,
		points: Array[PackedVector2Array]
) -> void:
	for polygon_points in points:
		tile_data.add_collision_polygon(layer_id)
		var polygon_index := tile_data.get_collision_polygons_count(layer_id) - 1
		tile_data.set_collision_polygon_points(layer_id, polygon_index, polygon_points)


func _full_tile_polygon(tileset: TileSet) -> PackedVector2Array:
	var half_size := Vector2(tileset.tile_size) * 0.5
	return PackedVector2Array([
		Vector2(-half_size.x, -half_size.y),
		Vector2(half_size.x, -half_size.y),
		Vector2(half_size.x, half_size.y),
		Vector2(-half_size.x, half_size.y),
	])
