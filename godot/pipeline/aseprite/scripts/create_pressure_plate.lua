-- Aseprite Script: Pressure Plate Generator
-- Creates a 20x16 pressure plate sprite

local SPRITE_WIDTH = 20
local SPRITE_HEIGHT = 16

-- Create new sprite
local sprite = Sprite(SPRITE_WIDTH, SPRITE_HEIGHT, ColorMode.RGB)
sprite.filename = "pressure_plate.aseprite"

-- Define colors
local stoneLight = Color({ r = 190, g = 190, b = 190, a = 255 })
local stoneMid = Color({ r = 120, g = 120, b = 125, a = 255 })
local stoneDark = Color({ r = 90, g = 90, b = 95, a = 255 })
local stoneBase = Color({ r = 60, g = 60, b = 70, a = 255 })
local activeAccent = Color({ r = 50, g = 230, b = 80, a = 255 })

-- Helper: Fill Rect
local function fillRect(cel, x, y, w, h, color)
	local img = cel.image
	for py = y, y + h - 1 do
		for px = x, x + w - 1 do
			if px >= 0 and px < img.width and py >= 0 and py < img.height then
				img:drawPixel(px, py, color)
			end
		end
	end
end

local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

-- Draw Base (Housing)
local function drawBase(cel)
	local baseY = SPRITE_HEIGHT - 3
	fillRect(cel, 0, baseY, SPRITE_WIDTH, 3, stoneBase)
	fillRect(cel, 0, baseY, SPRITE_WIDTH, 1, stoneDark) -- top edge shade
end

local function drawPlate(cel, pressed)
	drawBase(cel)

	local baseY = SPRITE_HEIGHT - 3
	local plateY = pressed and (baseY - 1) or (baseY - 3)
	local plateH = pressed and 2 or 3
	local topColor = pressed and stoneMid or stoneLight

	-- Shadow under raised plate to emphasize bump (no outline)
	if not pressed then
		fillRect(cel, 1, plateY + plateH, SPRITE_WIDTH - 2, 1, stoneDark)
	end

	-- Plate slab
	fillRect(cel, 0, plateY, SPRITE_WIDTH, plateH, topColor)
	-- Bevel: darker bottom edge
	fillRect(cel, 0, plateY + plateH - 1, SPRITE_WIDTH, 1, stoneMid)
	-- Slight side shading (keeps style consistent without black outline)
	for py = plateY, plateY + plateH - 1 do
		cel.image:drawPixel(0, py, stoneMid)
		cel.image:drawPixel(SPRITE_WIDTH - 1, py, stoneMid)
	end

	-- Small active indicator on the top edge corners
	if pressed then
		cel.image:drawPixel(1, plateY, activeAccent)
		cel.image:drawPixel(SPRITE_WIDTH - 2, plateY, activeAccent)
	end
end

local frameIndex = 1

-- Tag Helper
local tagDefs = {}
local function defineTag(name, fromFrame, toFrame)
	table.insert(tagDefs, { name = name, fromFrame = fromFrame, toFrame = toFrame })
end

local function newFrame()
	local frame
	if frameIndex == 1 then
		frame = sprite.frames[1]
	else
		frame = sprite:newEmptyFrame()
	end
	frameIndex = frameIndex + 1
	return frame
end

-- ==================== INACTIVE (Up) ====================
local inactiveStart = frameIndex
local frame = newFrame()
frame.duration = 1.0 -- Static
local cel = newCelWithImage(frame)

drawPlate(cel, false)

defineTag("inactive", inactiveStart, inactiveStart)

-- ==================== ACTIVE (Down) ====================
local activeStart = frameIndex
local frame2 = newFrame()
frame2.duration = 1.0 -- Static
local cel2 = newCelWithImage(frame2)

drawPlate(cel2, true)

defineTag("active", activeStart, activeStart)

for _, t in ipairs(tagDefs) do
	local tag = sprite:newTag(t.fromFrame, t.toFrame)
	tag.name = t.name
	tag.aniDir = AniDir.FORWARD
end

local outPath = app.params["out"]
if outPath ~= nil and outPath ~= "" then
	sprite:saveAs(outPath)
end

if app.isUIAvailable then
	app.alert("Pressure Plate Sprite Created!")
end
