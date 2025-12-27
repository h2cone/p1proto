@tool

# Entity Post-Import Script for LDtk Importer
# Automatically sets up entities during import based on their identifier

# Entities that require room_coords to be set
const ENTITIES_WITH_ROOM_COORDS := ["checkpoint", "plain_key", "plain_lock"]


func post_import(entity_layer: LDTKEntityLayer) -> LDTKEntityLayer:
	var entities: Array = entity_layer.entities

	print("Processing Entity Layer: ", entity_layer.name, " | Entity Count: ", entities.size())

	var entity_counts := { }
	for entity in entities:
		var entity_key := get_entity_key(entity)

		# Track entity sequence numbers
		if not entity_counts.has(entity_key):
			entity_counts[entity_key] = 0
		entity_counts[entity_key] += 1

		# Handle special entities with custom fields, otherwise use generic setup
		match entity_key:
			"moving_platform":
				setup_moving_platform(entity_layer, entity, entity_counts[entity_key])
			_:
				setup_generic_entity(entity_layer, entity, entity_counts[entity_key])

	return entity_layer


func get_entity_key(entity_data: Variant) -> String:
	"""Get lowercase entity identifier used for scene path lookup"""
	return get_entity_identifier(entity_data).to_lower()


func get_scene_path(entity_key: String) -> String:
	"""Build scene path from entity key"""
	return "res://entity/%s.tscn" % entity_key


func setup_generic_entity(entity_layer: LDTKEntityLayer, entity_data: Variant, sequence: int) -> void:
	"""Set up a generic entity without custom LDtk fields"""
	var entity_key := get_entity_key(entity_data)
	var scene_path := get_scene_path(entity_key)

	print("Setting up %s" % get_entity_identifier(entity_data))

	var instance := instantiate_entity(entity_layer, entity_data, scene_path, sequence)
	if not instance:
		return

	# Set room_coords for entities that need it
	if entity_key in ENTITIES_WITH_ROOM_COORDS:
		var room_coords: Variant = get_room_coords(entity_layer)
		if room_coords != null:
			instance.set("room_coords", room_coords)
		else:
			printerr("%s room coords could not be resolved for layer: %s" % [entity_key, entity_layer.name])

	finalize_entity(entity_layer, instance, entity_data, entity_key)
	print("  - Instantiated %s.tscn" % entity_key)


func instantiate_entity(_entity_layer: LDTKEntityLayer, entity_data: Variant, scene_path: String, sequence: int) -> Node:
	"""Load and instantiate entity scene with position and name"""
	var scene = load(scene_path)
	if not scene:
		printerr("Failed to load scene at: ", scene_path)
		return null

	var instance = scene.instantiate()
	instance.position = get_entity_anchor_position(entity_data, Vector2(0.5, 0.5))
	instance.name = build_entity_name(entity_data, sequence)
	return instance


func finalize_entity(entity_layer: LDTKEntityLayer, instance: Node, entity_data: Variant, entity_key: String) -> void:
	"""Add metadata and attach entity to scene tree"""
	var owner := resolve_owner(entity_layer)

	instance.set_meta("ldtk_iid", get_entity_iid(entity_data))
	instance.set_meta("ldtk_identifier", get_entity_identifier(entity_data))
	instance.set_meta("entity_type", entity_key)

	entity_layer.add_child(instance)
	set_owner_if_present(instance, owner)


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
	"""Set up a MovingPlatform entity with custom LDtk fields"""
	var entity_key := "moving_platform"
	var scene_path := get_scene_path(entity_key)

	print("Setting up %s" % get_entity_identifier(entity_data))

	var instance := instantiate_entity(entity_layer, entity_data, scene_path, sequence)
	if not instance:
		return

	# Read and apply custom fields from LDtk
	var travel_x: float = get_entity_field(entity_data, "travel_x", 96.0)
	var travel_y: float = get_entity_field(entity_data, "travel_y", 0.0)
	var duration: float = get_entity_field(entity_data, "duration", 2.0)
	var pause_time: float = get_entity_field(entity_data, "pause_time", 0.5)

	instance.set("travel", Vector2(travel_x, travel_y))
	instance.set("duration", duration)
	instance.set("pause_time", pause_time)

	finalize_entity(entity_layer, instance, entity_data, entity_key)
	print("  - Instantiated %s.tscn" % entity_key)
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
