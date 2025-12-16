-- Aseprite Script: Checkpoint Generator
-- Creates a 16x32 checkpoint animation

local SPRITE_WIDTH = 16
local SPRITE_HEIGHT = 32

-- Original art was authored for 16x28; keep the same silhouette and pad vertically to 32px.
local CONTENT_HEIGHT = 28
local Y_OFFSET = SPRITE_HEIGHT - CONTENT_HEIGHT

-- Create new sprite
local sprite = Sprite(SPRITE_WIDTH, SPRITE_HEIGHT, ColorMode.RGB)
sprite.filename = "checkpoint_flag.aseprite"

-- Define colors
local white = Color({ r = 255, g = 255, b = 255, a = 255 })
local grey = Color({ r = 100, g = 100, b = 100, a = 255 })
local darkGrey = Color({ r = 60, g = 60, b = 60, a = 255 })
local red = Color({ r = 230, g = 50, b = 50, a = 255 })
local darkRed = Color({ r = 150, g = 30, b = 30, a = 255 })
local green = Color({ r = 50, g = 230, b = 80, a = 255 })
local darkGreen = Color({ r = 30, g = 150, b = 50, a = 255 })
local highlight = Color({ r = 255, g = 255, b = 200, a = 100 }) -- transparent highlight

-- Helper function to draw a filled rectangle
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

-- Draw the pole and base (static elements)
local function drawPole(cel)
	local baseY = SPRITE_HEIGHT - 2
	-- Base
	fillRect(cel, 6, baseY, 4, 2, darkGrey)
	fillRect(cel, 7, baseY - 1, 2, 1, grey)

	-- Pole
	fillRect(cel, 7, 1 + Y_OFFSET, 2, CONTENT_HEIGHT - 3, white)
	-- Pole Topper
	fillRect(cel, 6, 0 + Y_OFFSET, 4, 1, white)
end

-- Draw the flag cloth
-- x, y: Top-left attachment point to the pole
-- color: Main color of the flag
-- waveOffset: Horizontal offset for waving effect (0, 1, or -1)
local function drawFlag(cel, y, color, shadeColor, waveOffset)
	local x = 9 -- Attached to right side of pole (pole is at x=7, width=2, so 7+2=9)
	local img = cel.image

	-- Flag shape (roughly rectangular with wave)
	local width = 7 -- Fits exactly from 9 to 15 (16px width)
	local height = 6

	for py = 0, height - 1 do
		for px = 0, width - 1 do
			-- Simple wave calculation
			local wave = 0
			if waveOffset ~= 0 then
				-- Shift pixels based on Y and offset
				if (py + px) % 3 == 0 then
					wave = waveOffset
				end
			end

			local drawX = x + px
			local drawY = y + py + wave

			-- Basic shading
			local c = color
			if py == height - 1 or px == width - 1 then
				c = shadeColor
			end

			-- Draw if within bounds
			if drawX < img.width and drawY < img.height then
				img:drawPixel(drawX, drawY, c)
			end
		end
	end
end

local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1

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

-- ==================== UNCHECKED (Idle) ====================
-- 1 frame: Red flag at bottom
local uncheckedStart = frameIndex
local frame = newFrame()
frame.duration = 0.5
local cel = newCelWithImage(frame)

drawPole(cel)
drawFlag(cel, 19 + Y_OFFSET, red, darkRed, 0) -- Low position

defineTag("unchecked", uncheckedStart, frameIndex - 1)

-- ==================== RAISING (Transition) ====================
-- Flag moves up and turns green
local raisingStart = frameIndex
local steps = 6
local startY = 19 + Y_OFFSET
local endY = 1 + Y_OFFSET

for i = 0, steps - 1 do
	local frame = newFrame()
	frame.duration = 0.08
	local cel = newCelWithImage(frame)

	drawPole(cel)

	-- Linear interpolation for position
	local t = i / (steps - 1)
	local currentY = math.floor(startY + (endY - startY) * t)

	-- Color transition: Red until halfway, then Green
	local currentColor = (t < 0.5) and red or green
	local currentShade = (t < 0.5) and darkRed or darkGreen

	drawFlag(cel, currentY, currentColor, currentShade, (i % 2 == 0) and 1 or -1)
end

defineTag("raising", raisingStart, frameIndex - 1)

-- ==================== CHECKED (Active) ====================
-- Loop: Green flag waving at top
local checkedStart = frameIndex
for i = 1, 4 do
	local frame = newFrame()
	frame.duration = 0.15
	local cel = newCelWithImage(frame)

	drawPole(cel)

	-- Waving animation
	local wave = 0
	if i == 1 then
		wave = 0
	elseif i == 2 then
		wave = 1
	elseif i == 3 then
		wave = 0
	elseif i == 4 then
		wave = -1
	end

	drawFlag(cel, 1 + Y_OFFSET, green, darkGreen, wave)
end

defineTag("checked", checkedStart, frameIndex - 1)

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
	app.alert("Checkpoint Sprite Created!")
end
