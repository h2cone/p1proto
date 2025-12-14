-- Aseprite Script: Pressure Plate Generator
-- Creates a 16x16 pressure plate sprite

-- Create new sprite
local sprite = Sprite(16, 16, ColorMode.RGB)
sprite.filename = "pressure_plate.aseprite"

-- Define colors
local stoneLight = Color({ r = 180, g = 180, b = 180, a = 255 })
local stoneDark = Color({ r = 100, g = 100, b = 100, a = 255 })
local stoneBase = Color({ r = 60, g = 60, b = 70, a = 255 })
local outline = Color({ r = 30, g = 30, b = 35, a = 255 })

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

-- Helper: Draw Rect Outline
local function drawRect(cel, x, y, w, h, color)
	local img = cel.image
	-- Top/Bottom
	for px = x, x + w - 1 do
		if y >= 0 and y < img.height then
			img:drawPixel(px, y, color)
		end
		if y + h - 1 >= 0 and y + h - 1 < img.height then
			img:drawPixel(px, y + h - 1, color)
		end
	end
	-- Left/Right
	for py = y, y + h - 1 do
		if x >= 0 and x < img.width then
			img:drawPixel(x, py, color)
		end
		if x + w - 1 >= 0 and x + w - 1 < img.width then
			img:drawPixel(x + w - 1, py, color)
		end
	end
end

local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

-- Draw Base (Housing)
local function drawBase(cel)
	-- Base is a wider, dark platform
	-- x=1, y=13, w=14, h=3 (bottom 3 pixels)
	fillRect(cel, 1, 13, 14, 3, stoneBase)
	drawRect(cel, 1, 13, 14, 3, outline)
end

-- Draw Plate Up (Inactive)
local function drawPlateUp(cel)
	drawBase(cel)
	-- Plate is raised: x=3, y=10, w=10, h=3
	-- Sits on top of base (y=13)
	local px, py, pw, ph = 3, 10, 10, 3

	fillRect(cel, px, py, pw, ph, stoneLight)
	drawRect(cel, px, py, pw, ph, outline)

	-- Side shading (makes it look 3D)
	for px_i = px + 1, px + pw - 2 do
		cel.image:drawPixel(px_i, py + ph - 1, stoneDark)
	end
end

-- Draw Plate Down (Active)
local function drawPlateDown(cel)
	drawBase(cel)
	-- Plate is pushed down: x=3, y=12, w=10, h=2
	-- Sits inside base (y=13 overlaps)
	local px, py, pw, ph = 3, 12, 10, 2

	fillRect(cel, px, py, pw, ph, stoneLight)
	drawRect(cel, px, py, pw, ph, outline)

	-- No side shading, it's flat against the base
end

local frameIndex = 1

-- Tag Helper
local function createTag(name, fromFrame, toFrame)
	sprite:newTag(fromFrame, toFrame)
	sprite.tags[#sprite.tags].name = name
end

-- ==================== INACTIVE (Up) ====================
local inactiveStart = frameIndex
local frame = sprite:newEmptyFrame(frameIndex)
frameIndex = frameIndex + 1
frame.duration = 1.0 -- Static
local cel = newCelWithImage(frame)

drawPlateUp(cel)

createTag("inactive", inactiveStart, inactiveStart)

-- ==================== ACTIVE (Down) ====================
local activeStart = frameIndex
local frame2 = sprite:newEmptyFrame(frameIndex)
frameIndex = frameIndex + 1
frame2.duration = 1.0 -- Static
local cel2 = newCelWithImage(frame2)

drawPlateDown(cel2)

createTag("active", activeStart, activeStart)

app.alert("Pressure Plate Sprite Created!")
