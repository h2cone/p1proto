@tool

const CHECKPOINT_SCENE_PATH := "res://checkpoint/checkpoint.tscn"
const CHECKPOINT_SPRITE_PATH := "res://pipeline/aseprite/checkpoint_flag.res"
const MOVING_PLATFORM_SCENE_PATH := "res://platforms/moving_platform.tscn"

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
			"CheckpointFlag":
				checkpoint_count += 1
				setup_checkpoint(entity_layer, entity, checkpoint_count)
			"MovingPlatform":
				platform_count += 1
				setup_moving_platform(entity_layer, entity, platform_count)
			_:
				pass

	return entity_layer


func setup_checkpoint(entity_layer: LDTKEntityLayer, entity_data: Variant, sequence: int) -> void:
	"""Set up a CheckpointFlag entity with all required components"""

	print("Setting up Checkpoint: ", get_entity_identifier(entity_data))

	var owner := resolve_owner(entity_layer)
	var anchor := ensure_entity_anchor(entity_layer, entity_data, sequence, owner)
	anchor.set_meta("entity_type", "checkpoint")

	# Load checkpoint scene
	var checkpoint_scene = load(CHECKPOINT_SCENE_PATH)
	if not checkpoint_scene:
		printerr("Failed to load checkpoint.tscn - using manual setup")
		setup_checkpoint_manual(anchor, owner)
		return

	# Instance the checkpoint scene
	var checkpoint_instance = checkpoint_scene.instantiate()
	checkpoint_instance.name = "Checkpoint"

	var room_coords: Variant = get_room_coords(entity_layer)
	if room_coords != null:
		checkpoint_instance.set("room_coords", room_coords)
	else:
		printerr("Checkpoint room coords could not be resolved for layer: ", entity_layer.name)

	# Add as child of the entity anchor
	anchor.add_child(checkpoint_instance)
	set_owner_if_present(checkpoint_instance, owner)

	print("  - Instantiated checkpoint.tscn as child")


func ensure_entity_anchor(
		entity_layer: LDTKEntityLayer,
		entity_data: Variant,
		sequence: int,
		owner: Node,
) -> Node2D:
	# Importer stores entities as Dictionaries by default; if a placeholder already exists, reuse it.
	if entity_data is LDTKEntity:
		set_owner_if_present(entity_data, owner)
		return entity_data

	var anchor := Node2D.new()
	anchor.position = get_entity_position(entity_data)
	anchor.name = build_entity_name(entity_data, sequence)
	anchor.set_meta("ldtk_iid", get_entity_iid(entity_data))
	anchor.set_meta("ldtk_identifier", get_entity_identifier(entity_data))

	entity_layer.add_child(anchor)
	set_owner_if_present(anchor, owner)
	return anchor


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


func setup_checkpoint_manual(parent: Node, owner: Node) -> void:
	"""Manually create checkpoint structure if checkpoint.tscn doesn't exist"""

	# 1. Add AnimatedSprite2D with checkpoint_flag.res
	var sprite := AnimatedSprite2D.new()
	sprite.name = "AnimatedSprite2D"

	# Load the SpriteFrames resource
	var sprite_frames = load(CHECKPOINT_SPRITE_PATH) as SpriteFrames
	if sprite_frames:
		sprite.sprite_frames = sprite_frames
		sprite.animation = "unchecked"
		sprite.autoplay = "" # Don't autoplay, let the script control it
	else:
		printerr("Failed to load checkpoint_flag.res")

	# Center the sprite on the entity (entity size is 16x24)
	sprite.offset = Vector2(8, 12)

	parent.add_child(sprite)
	set_owner_if_present(sprite, owner)

	# 2. Add CollisionShape2D for player detection
	var collision := CollisionShape2D.new()
	collision.name = "CollisionShape2D"

	# Create a rectangle shape matching the entity size
	var shape := RectangleShape2D.new()
	shape.size = Vector2(16, 24)
	collision.shape = shape

	# Center the collision shape
	collision.position = Vector2(8, 12)

	parent.add_child(collision)
	set_owner_if_present(collision, owner)

	print("  - Added AnimatedSprite2D with checkpoint_flag.res")
	print("  - Added CollisionShape2D (16x24)")
	print("  - Note: checkpoint.tscn not found, using manual setup")
	print("  - Attach Checkpoint script at runtime or create checkpoint.tscn")


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
		printerr("Failed to load moving_platform.tscn - using manual setup")
		var platform_width: int = int(get_entity_field(entity_data, "platform_width", 32))
		var platform_height: int = int(get_entity_field(entity_data, "platform_height", 8))
		setup_moving_platform_manual(
			entity_layer,
			entity_data,
			sequence,
			owner,
			travel_x,
			travel_y,
			duration,
			pause_time,
			platform_width,
			platform_height,
		)
		return

	# Instance the platform scene
	var platform = platform_scene.instantiate()
	platform.position = get_entity_position(entity_data)
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


func setup_moving_platform_manual(
		entity_layer: LDTKEntityLayer,
		entity_data: Variant,
		sequence: int,
		owner: Node,
		travel_x: float,
		travel_y: float,
		duration: float,
		pause_time: float,
		platform_width: int,
		platform_height: int,
) -> void:
	"""Manually create MovingPlatform structure if moving_platform.tscn doesn't exist"""

	# Create the MovingPlatform node (Rust GDExtension class)
	var platform = ClassDB.instantiate("MovingPlatform")
	if not platform:
		printerr("Failed to instantiate MovingPlatform - is the Rust extension loaded?")
		return

	# Set position
	platform.position = get_entity_position(entity_data)
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

	# Add visual representation (Sprite2D)
	var sprite := Sprite2D.new()
	sprite.name = "Sprite2D"
	sprite.texture = create_platform_texture(platform_width, platform_height)
	sprite.centered = false
	sprite.offset = Vector2(-platform_width / 2.0, -platform_height / 2.0)
	platform.add_child(sprite)
	set_owner_if_present(sprite, owner)

	# Add collision shape
	var collision := CollisionShape2D.new()
	collision.name = "CollisionShape2D"
	var shape := RectangleShape2D.new()
	shape.size = Vector2(platform_width, platform_height)
	collision.shape = shape
	platform.add_child(collision)
	set_owner_if_present(collision, owner)

	print("  - Created MovingPlatform manually: travel=(%.1f, %.1f), duration=%.1fs, pause=%.1fs" % [travel_x, travel_y, duration, pause_time])
	print("  - Added Sprite2D (%dx%d) and CollisionShape2D" % [platform_width, platform_height])
	print("  - Note: moving_platform.tscn not found, using manual setup")


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


func create_platform_texture(width: int, height: int) -> ImageTexture:
	"""Create a simple colored texture for the platform"""
	var image := Image.create(width, height, false, Image.FORMAT_RGBA8)
	image.fill(Color(0.8, 0.6, 0.4, 1.0)) # Brown/tan color

	# Add border for visibility
	for x in width:
		image.set_pixel(x, 0, Color.BLACK)
		image.set_pixel(x, height - 1, Color.BLACK)
	for y in height:
		image.set_pixel(0, y, Color.BLACK)
		image.set_pixel(width - 1, y, Color.BLACK)

	return ImageTexture.create_from_image(image)
