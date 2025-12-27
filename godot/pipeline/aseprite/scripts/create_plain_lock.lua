-- Aseprite Script: Lock Sprite Generator
-- Creates a 32x32 lock block (larger than 16x24 player)

local sprite = Sprite(32, 32, ColorMode.RGB)
sprite.filename = "plain_lock.aseprite"

-- Palette
local goldLight = Color({ r = 255, g = 215, b = 0, a = 255 })
local goldShadow = Color({ r = 184, g = 134, b = 11, a = 255 })
local metalGrey = Color({ r = 169, g = 169, b = 169, a = 255 })
local metalLight = Color({ r = 211, g = 211, b = 211, a = 255 })
local black = Color({ r = 0, g = 0, b = 0, a = 255 })

-- Helper: Fill Rect
local function fillRect(img, x, y, w, h, color)
	for py = y, y + h - 1 do
		for px = x, x + w - 1 do
			if px >= 0 and px < img.width and py >= 0 and py < img.height then
				img:drawPixel(px, py, color)
			end
		end
	end
end

-- Helper: Outline Rect
local function outlineRect(img, x, y, w, h, color)
	fillRect(img, x, y, w, 1, color) -- Top
	fillRect(img, x, y + h - 1, w, 1, color) -- Bottom
	fillRect(img, x, y, 1, h, color) -- Left
	fillRect(img, x + w - 1, y, 1, h, color) -- Right
end

-- Draw Lock
local cel = sprite.cels[1]
local img = cel.image

-- Main Body (Slightly smaller than full 32x32 to have depth/edge)
-- Let's make it a 30x30 block centered
local bx, by = 1, 1
local bw, bh = 30, 30

-- 1. Base Gold Block
fillRect(img, bx, by, bw, bh, goldLight)

-- 2. Shading (Bevel effect)
-- Right and Bottom shadow
fillRect(img, bx + bw - 2, by, 2, bh, goldShadow)
fillRect(img, bx, by + bh - 2, bw, 2, goldShadow)
-- Outline
outlineRect(img, bx, by, bw, bh, goldShadow)

-- 3. Keyhole (Center)
-- A circle top + trapezoid bottom
local cx, cy = 16, 16
-- Circle part (approx radius 3)
fillRect(img, cx - 2, cy - 4, 4, 4, black)
-- Shaft part
fillRect(img, cx - 1, cy - 1, 2, 6, black)

-- 4. Screws (Corners)
local screwSize = 4
local offset = 3
local screwColor = metalGrey
local screwHighlight = metalLight

local function drawScrew(sx, sy)
	fillRect(img, sx, sy, screwSize, screwSize, screwColor)
	-- Screw slot (diagonal or straight)
	img:drawPixel(sx + 1, sy + 1, screwHighlight)
	img:drawPixel(sx + 2, sy + 2, screwHighlight)
end

drawScrew(bx + offset, by + offset) -- Top Left
drawScrew(bx + bw - offset - screwSize, by + offset) -- Top Right
drawScrew(bx + offset, by + bh - offset - screwSize) -- Bottom Left
drawScrew(bx + bw - offset - screwSize, by + bh - offset - screwSize) -- Bottom Right

-- Add a "Static" tag just in case
local tag = sprite:newTag(1, 1)
tag.name = "default"

app.alert("Lock Sprite Created!")
