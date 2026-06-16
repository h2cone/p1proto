@tool
extends RefCounted

const ENTITY_POST_IMPORT_SCRIPT := preload("res://pipeline/ldtk/entities_post_import.gd")
const BACK_DECOR_LAYER_NAME := "BackDecor"
const BACK_DECOR_Z_INDEX := -20


func post_import(level: Node) -> Node:
	_normalize_background_layers(level)
	_populate_entity_layers(level)
	return level


func _normalize_background_layers(level: Node) -> void:
	for child in level.get_children():
		if child.name != BACK_DECOR_LAYER_NAME:
			continue
		if not child is TileMapLayer:
			continue

		var layer: TileMapLayer = child
		layer.z_index = BACK_DECOR_Z_INDEX
		_normalize_background_tile_layer(layer)


func _normalize_background_tile_layer(layer: TileMapLayer) -> void:
	layer.y_sort_enabled = false
	layer.collision_enabled = false
	layer.navigation_enabled = false

	for child in layer.get_children():
		if child is TileMapLayer:
			_normalize_background_tile_layer(child)


func _populate_entity_layers(level: Node) -> void:
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
