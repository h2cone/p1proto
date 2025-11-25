@tool
extends AnimatedSprite2D

## 在编辑器中预览动画
## 设置 animation 属性后会自动播放

@export var preview_in_editor: bool = false:
	set(value):
		preview_in_editor = value
		if Engine.is_editor_hint():
			if value and animation != "":
				play(animation)
			else:
				stop()

func _ready():
	if Engine.is_editor_hint() and preview_in_editor and animation != "":
		play(animation)
