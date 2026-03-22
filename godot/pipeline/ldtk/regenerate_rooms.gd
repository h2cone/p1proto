@tool
extends SceneTree

const LEVELS_DIR := "res://pipeline/ldtk/levels"
const ENTITY_POST_IMPORT_SCRIPT := preload("res://pipeline/ldtk/entities_post_import.gd")


func _init() -> void:
	var entity_post_import = ENTITY_POST_IMPORT_SCRIPT.new()
	var dir := DirAccess.open(LEVELS_DIR)
	if dir == null:
		push_error("Failed to open levels dir: %s" % LEVELS_DIR)
		quit(1)
		return

	var updated_rooms := 0
	for file_name in dir.get_files():
		if not file_name.begins_with("Room_") or not file_name.ends_with(".scn"):
			continue

		var room_path := "%s/%s" % [LEVELS_DIR, file_name]
		var room_scene: PackedScene = load(room_path)
		if room_scene == null:
			push_error("Failed to load room scene: %s" % room_path)
			quit(1)
			return

		var room: Node = room_scene.instantiate()
		var changed := false

		for child in room.get_children():
			if child is not LDTKEntityLayer:
				continue

			var entity_layer: LDTKEntityLayer = child
			if entity_layer.entities.is_empty():
				continue

			_clear_entity_layer_children(entity_layer)

			entity_post_import.post_import(entity_layer)
			changed = true

		if not changed:
			continue

		for child in room.get_children():
			_set_owner_recursive(child, room)

		var packed := PackedScene.new()
		var pack_err := packed.pack(room)
		if pack_err != OK:
			push_error("Failed to pack room scene: %s (%s)" % [room_path, pack_err])
			quit(1)
			return

		var save_err := ResourceSaver.save(packed, room_path)
		if save_err != OK:
			push_error("Failed to save room scene: %s (%s)" % [room_path, save_err])
			quit(1)
			return

		updated_rooms += 1
		print("Regenerated ", room_path)

	print("Updated rooms: ", updated_rooms)
	quit()


func _clear_entity_layer_children(entity_layer: LDTKEntityLayer) -> void:
	for child in entity_layer.get_children():
		entity_layer.remove_child(child)
		child.owner = null
		child.free()


func _set_owner_recursive(node: Node, owner: Node) -> void:
	node.owner = owner
	for child in node.get_children():
		_set_owner_recursive(child, owner)
