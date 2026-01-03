-- Aseprite Script: Pushable Crate Generator
-- Creates a 24x24 wooden crate (height is multiple of 8, matches player height)

local sprite = Sprite(24, 24, ColorMode.RGB)
sprite.filename = "pushable_crate.aseprite"

-- Palette (Warm wooden tones to harmonize with vibrant world)
local woodLight = Color({ r = 220, g = 170, b = 110, a = 255 })
local woodMain = Color({ r = 180, g = 130, b = 80, a = 255 })
local woodShadow = Color({ r = 140, g = 90, b = 50, a = 255 })
local outlineColor = Color({ r = 90, g = 50, b = 30, a = 255 }) -- Dark brown outline

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

-- Helper: Draw Rect (Outline)
local function drawRect(img, x, y, w, h, color)
	fillRect(img, x, y, w, 1, color) -- Top
	fillRect(img, x, y + h - 1, w, 1, color) -- Bottom
	fillRect(img, x, y, 1, h, color) -- Left
	fillRect(img, x + w - 1, y, 1, h, color) -- Right
end

local cel = sprite.cels[1]
local img = cel.image

-- 1. Main Body Fill
fillRect(img, 1, 1, 22, 22, woodMain)

-- 2. Inner Frame/Bevel
-- Top and Left Highlight
fillRect(img, 2, 2, 20, 2, woodLight)
fillRect(img, 2, 2, 2, 20, woodLight)
-- Bottom and Right Shadow
fillRect(img, 20, 2, 2, 20, woodShadow)
fillRect(img, 2, 20, 20, 2, woodShadow)

-- 3. Cross Brace (X style) for structural crate look
-- Center the X within the frame (approx 4 to 19)
local function drawDiagonal(img, x1, y1, x2, y2, color)
	-- Simple line algo or direct pixel setting for this specific size
	-- Drawing thick diagonals for the crate brace
	local steps = math.abs(x2 - x1)
	for i = 0, steps do
		local t = i / steps
		local x = math.floor(x1 + (x2 - x1) * t + 0.5)
		local y = math.floor(y1 + (y2 - y1) * t + 0.5)
		img:drawPixel(x, y, color)
		img:drawPixel(x + 1, y, color) -- Make it slightly thicker
	end
end

-- Diagonal 1: Top-Left to Bottom-Right
drawDiagonal(img, 5, 5, 18, 18, woodShadow)
-- Diagonal 2: Bottom-Left to Top-Right
drawDiagonal(img, 5, 18, 18, 5, woodShadow)

-- 4. Outer Outline (1px)
drawRect(img, 0, 0, 24, 24, outlineColor)

-- 5. Corner Reinforcements (Metal or Darker Wood)
local function drawCorner(cx, cy)
	-- 4x4 Corner bracket
	fillRect(img, cx, cy, 5, 5, outlineColor)
	-- Rivet/Highlight inside
	img:drawPixel(cx + 1, cy + 1, woodLight)
	if cx > 10 then -- Adjust highlight for right side
		img:drawPixel(cx + 1, cy + 1, woodLight)
	end
end

-- Draw corners outside the main fill slightly or overlapping
-- Top Left
fillRect(img, 0, 0, 5, 5, outlineColor)
img:drawPixel(1, 1, woodLight)
img:drawPixel(3, 1, woodMain)
img:drawPixel(1, 3, woodMain)

-- Top Right
fillRect(img, 19, 0, 5, 5, outlineColor)
img:drawPixel(20, 1, woodLight)

-- Bottom Left
fillRect(img, 0, 19, 5, 5, outlineColor)
img:drawPixel(1, 20, woodLight)

-- Bottom Right
fillRect(img, 19, 19, 5, 5, outlineColor)
img:drawPixel(20, 20, woodLight)

-- Tag
local tag = sprite:newTag(1, 1)
tag.name = "default"

app.alert("Pushable Crate Sprite Created!")
