@tool
extends RefCounted

const ENTITY_POST_IMPORT_SCRIPT := preload("res://pipeline/ldtk/entities_post_import.gd")


func post_import(level: Node) -> Node:
	var entity_post_import = ENTITY_POST_IMPORT_SCRIPT.new()

	for child in level.get_children():
		if not child is LDTKEntityLayer:
			continue

		var entity_layer: LDTKEntityLayer = child
		if entity_layer.entities.is_empty():
			continue
		if entity_layer.get_child_count() > 0:
			continue

		entity_post_import.post_import(entity_layer)

	return level
