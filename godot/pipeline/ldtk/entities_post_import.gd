@tool

const CHECKPOINT_SCENE_PATH := "res://entity/checkpoint.tscn"
const MOVING_PLATFORM_SCENE_PATH := "res://entity/moving_platform.tscn"

# Entity Post-Import Script for LDtk Importer
# Automatically sets up entities during import based on their identifier


func post_import(entity_layer: LDTKEntityLayer) -> LDTKEntityLayer:
	var entities: Array = entity_layer.entities

	print("Processing Entity Layer: ", entity_layer.name, " | Entity Count: ", entities.size())

	var checkpoint_count := 0
	var platform_count := 0
	for entity in entities:
		var entity_identifier := get_entity_identifier(entity)
		match entity_identifier:
			"Checkpoint":
				checkpoint_count += 1
				setup_checkpoint(entity_layer, entity, checkpoint_count)
			"MovingPlatform":
				platform_count += 1
				setup_moving_platform(entity_layer, entity, platform_count)
			_:
				pass

	return entity_layer


func setup_checkpoint(entity_layer: LDTKEntityLayer, entity_data: Variant, sequence: int) -> void:
	"""Set up a Checkpoint entity with all required components"""

	print("Setting up Checkpoint: ", get_entity_identifier(entity_data))

	var owner := resolve_owner(entity_layer)

	# Load checkpoint scene
	var checkpoint_scene = load(CHECKPOINT_SCENE_PATH)
	if not checkpoint_scene:
		printerr("Failed to load checkpoint.tscn at: ", CHECKPOINT_SCENE_PATH)
		return

	# Instance the checkpoint scene
	var checkpoint = checkpoint_scene.instantiate()
	checkpoint.position = get_entity_anchor_position(entity_data, Vector2(0.5, 0.5))
	checkpoint.name = build_entity_name(entity_data, sequence)

	# Set room coordinates
	var room_coords: Variant = get_room_coords(entity_layer)
	if room_coords != null:
		checkpoint.set("room_coords", room_coords)
	else:
		printerr("Checkpoint room coords could not be resolved for layer: ", entity_layer.name)

	# Add metadata
	checkpoint.set_meta("ldtk_iid", get_entity_iid(entity_data))
	checkpoint.set_meta("ldtk_identifier", get_entity_identifier(entity_data))
	checkpoint.set_meta("entity_type", "checkpoint")

	# Add to scene tree
	entity_layer.add_child(checkpoint)
	set_owner_if_present(checkpoint, owner)

	print("  - Instantiated checkpoint.tscn")


func get_entity_identifier(entity_data: Variant) -> String:
	if entity_data is Dictionary:
		if entity_data.has("identifier"):
			return str(entity_data["identifier"])
		if entity_data.has("definition") and entity_data["definition"] is Dictionary:
			return str(entity_data["definition"].get("identifier", ""))
	if entity_data is LDTKEntity:
		return entity_data.identifier
	return ""


func get_entity_iid(entity_data: Variant) -> String:
	if entity_data is Dictionary and entity_data.has("iid"):
		return str(entity_data["iid"])
	if entity_data is LDTKEntity:
		return entity_data.iid
	return ""


func build_entity_name(entity_data: Variant, sequence: int) -> String:
	var base := get_entity_identifier(entity_data)
	if sequence > 1:
		return "%s%d" % [base, sequence]
	return base


func get_entity_position(entity_data: Variant) -> Vector2:
	if entity_data is Dictionary and entity_data.has("position"):
		var pos = entity_data["position"]
		if pos is Vector2 or pos is Vector2i:
			return Vector2(pos.x, pos.y)
	if entity_data is LDTKEntity:
		return Vector2(entity_data.position)
	return Vector2.ZERO


func get_entity_size(entity_data: Variant) -> Vector2:
	if entity_data is Dictionary and entity_data.has("size"):
		var size = entity_data["size"]
		if size is Vector2 or size is Vector2i:
			return Vector2(size.x, size.y)
	if entity_data is LDTKEntity:
		return Vector2(entity_data.size)
	return Vector2.ZERO


func get_entity_anchor_position(entity_data: Variant, anchor: Vector2) -> Vector2:
	# Imported LDtk entities use a top-left position and provide a size. This helper converts that
	# rectangle into a desired anchor point (e.g. center = (0.5, 0.5)).
	var pos := get_entity_position(entity_data)
	var size := get_entity_size(entity_data)
	if size == Vector2.ZERO:
		return pos
	return pos + (size * anchor)


func get_room_coords(entity_layer: LDTKEntityLayer) -> Variant:
	var level := entity_layer.get_parent()
	if level:
		var level_name: String = String(level.name)
		var coords: Variant = parse_room_name(level_name)
		if coords != null:
			return coords
	return null


func parse_room_name(name: String) -> Variant:
	var prefix := "Room_"
	if not name.begins_with(prefix):
		return null

	var parts := name.substr(prefix.length()).split("_")
	if parts.size() != 2:
		return null

	if not parts[0].is_valid_int() or not parts[1].is_valid_int():
		return null

	return Vector2i(int(parts[0]), int(parts[1]))


func resolve_owner(node: Node) -> Node:
	if node.owner:
		return node.owner
	var tree := node.get_tree()
	if tree:
		return tree.edited_scene_root if Engine.is_editor_hint() else tree.current_scene
	return null


func set_owner_if_present(node: Node, owner: Node) -> void:
	if owner:
		node.owner = owner


func setup_moving_platform(entity_layer: LDTKEntityLayer, entity_data: Variant, sequence: int) -> void:
	"""Set up a MovingPlatform entity with all required components"""

	print("Setting up MovingPlatform: ", get_entity_identifier(entity_data))

	var owner := resolve_owner(entity_layer)

	# Read custom fields from LDtk
	var travel_x: float = get_entity_field(entity_data, "travel_x", 96.0)
	var travel_y: float = get_entity_field(entity_data, "travel_y", 0.0)
	var duration: float = get_entity_field(entity_data, "duration", 2.0)
	var pause_time: float = get_entity_field(entity_data, "pause_time", 0.5)

	# Try to load the prefab scene
	var platform_scene = load(MOVING_PLATFORM_SCENE_PATH)
	if not platform_scene:
		printerr("Failed to load moving_platform.tscn at: ", MOVING_PLATFORM_SCENE_PATH)
		return

	# Instance the platform scene
	var platform = platform_scene.instantiate()
	platform.position = get_entity_anchor_position(entity_data, Vector2(0.5, 0.5))
	platform.name = build_entity_name(entity_data, sequence)

	# Set MovingPlatform properties
	platform.set("travel", Vector2(travel_x, travel_y))
	platform.set("duration", duration)
	platform.set("pause_time", pause_time)

	# Add metadata
	platform.set_meta("ldtk_iid", get_entity_iid(entity_data))
	platform.set_meta("ldtk_identifier", get_entity_identifier(entity_data))
	platform.set_meta("entity_type", "moving_platform")

	# Add to scene tree
	entity_layer.add_child(platform)
	set_owner_if_present(platform, owner)

	print("  - Instantiated moving_platform.tscn")
	print("  - Configured: travel=(%.1f, %.1f), duration=%.1fs, pause=%.1fs" % [travel_x, travel_y, duration, pause_time])


func get_entity_field(entity_data: Variant, field_name: String, default_value: Variant) -> Variant:
	"""Get a field value from entity data, with fallback to default"""
	if entity_data is Dictionary and entity_data.has("fields"):
		var fields = entity_data["fields"]
		if fields is Dictionary and fields.has(field_name):
			return fields[field_name]
	if entity_data is LDTKEntity:
		if entity_data.fields.has(field_name):
			return entity_data.fields[field_name]
	return default_value
