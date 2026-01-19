-- Aseprite Script: One-Way Platform Generator
-- Creates a 32x8 one-way platform
-- Design: solid top surface + downward-pointing teeth (inverted trapezoids)
-- This visually suggests: stand on top, pass through from below

local SPRITE_WIDTH = 32
local SPRITE_HEIGHT = 8

-- Create new sprite
local sprite = Sprite(SPRITE_WIDTH, SPRITE_HEIGHT, ColorMode.RGB)
sprite.filename = "one_way_platform.aseprite"

-- Define colors - light colors
local topHighlight = Color({ r = 255, g = 250, b = 240, a = 255 }) -- Top edge highlight
local topSurface = Color({ r = 235, g = 225, b = 210, a = 255 }) -- Standing surface
local toothFill = Color({ r = 220, g = 210, b = 195, a = 255 }) -- Tooth fill
local toothEdge = Color({ r = 190, g = 175, b = 155, a = 255 }) -- Tooth edge/bottom

-- Configuration
local TOOTH_WIDTH = 8 -- Each tooth is 8px wide at top
local NUM_TEETH = SPRITE_WIDTH / TOOTH_WIDTH -- 4 teeth

-- Helper: Draw single pixel safely
local function drawPixel(img, x, y, color)
	if x >= 0 and x < img.width and y >= 0 and y < img.height then
		img:drawPixel(x, y, color)
	end
end

-- Helper: Fill horizontal line
local function fillLine(img, x, y, w, color)
	for i = 0, w - 1 do
		drawPixel(img, x + i, y, color)
	end
end

-- Draw the one-way platform
-- Structure: solid top + 4 inverted isoceles trapezoid teeth
--   Row 0:   ████████████████████████████████  (top highlight)
--   Row 1:   ████████████████████████████████  (top surface)
--   Row 2:   ████████████████████████████████  (trapezoid tops, 8px each, continuous)
--   Row 3:   ████████████████████████████████  (8px, continuous)
--   Row 4:    ██████  ██████  ██████  ██████   (6px, indent 1 each side)
--   Row 5:    ██████  ██████  ██████  ██████   (6px)
--   Row 6:     ████    ████    ████    ████    (4px, indent 2 each side)
--   Row 7:     ████    ████    ████    ████    (4px, bottom)
--
local function drawPlatform(cel)
	local img = cel.image

	-- Row 0: Top highlight (continuous)
	fillLine(img, 0, 0, SPRITE_WIDTH, topHighlight)

	-- Row 1: Top surface (continuous)
	fillLine(img, 0, 1, SPRITE_WIDTH, topSurface)

	-- Row 2-3: Trapezoid tops (8px each, continuous)
	fillLine(img, 0, 2, SPRITE_WIDTH, toothFill)
	fillLine(img, 0, 3, SPRITE_WIDTH, toothFill)

	-- Row 4-7: Trapezoid bodies (4 teeth, each narrows symmetrically)
	for t = 0, NUM_TEETH - 1 do
		local baseX = t * TOOTH_WIDTH -- 0, 8, 16, 24

		-- Row 4-5: 6px wide, centered (indent 1 on each side)
		fillLine(img, baseX + 1, 4, 6, toothFill)
		fillLine(img, baseX + 1, 5, 6, toothFill)

		-- Row 6: 4px wide, centered (indent 2 on each side)
		fillLine(img, baseX + 2, 6, 4, toothFill)

		-- Row 7: 4px wide, bottom edge
		fillLine(img, baseX + 2, 7, 4, toothEdge)
	end
end

-- Frame helper functions
local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1

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

-- Tag definitions
local tagDefs = {}
local function defineTag(name, fromFrame, toFrame)
	table.insert(tagDefs, { name = name, fromFrame = fromFrame, toFrame = toFrame })
end

-- ==================== DEFAULT (Static) ====================
local defaultStart = frameIndex
local frame = newFrame()
frame.duration = 1.0
local cel = newCelWithImage(frame)

drawPlatform(cel)

defineTag("default", defaultStart, defaultStart)

-- Create tags
for _, t in ipairs(tagDefs) do
	local tag = sprite:newTag(t.fromFrame, t.toFrame)
	tag.name = t.name
	tag.aniDir = AniDir.FORWARD
end

-- Save if output path provided
local outPath = app.params["out"]
if outPath ~= nil and outPath ~= "" then
	sprite:saveAs(outPath)
end

if app.isUIAvailable then
	app.alert("One-Way Platform Sprite Created!")
end
