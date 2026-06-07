-- Aseprite Script: Ladder Generator
-- Creates a 16px wide ladder preview with an 8px rung rhythm.
-- Optional batch param: height=<pixels>. Height is snapped to the rung rhythm.

local DEFAULT_HEIGHT = 64
local SPRITE_WIDTH = 16
local MIN_HEIGHT = 24
local RUNG_PITCH = 8

local function readNumberParam(name, defaultValue)
	if app.params == nil then
		return defaultValue
	end

	local rawValue = app.params[name]
	if rawValue == nil or rawValue == "" then
		return defaultValue
	end

	local numberValue = tonumber(rawValue)
	if numberValue == nil then
		return defaultValue
	end

	return numberValue
end

local function snapToRungPitch(value)
	local snapped = math.floor((value / RUNG_PITCH) + 0.5) * RUNG_PITCH
	if snapped < MIN_HEIGHT then
		return MIN_HEIGHT
	end
	return snapped
end

local SPRITE_HEIGHT = snapToRungPitch(readNumberParam("height", DEFAULT_HEIGHT))

local sprite = Sprite(SPRITE_WIDTH, SPRITE_HEIGHT, ColorMode.RGB)
sprite.filename = "ladder.aseprite"

-- Palette: warm wood, close to the crate/platform family but slightly darker for readability.
local woodHighlight = Color({ r = 235, g = 190, b = 125, a = 255 })
local woodMain = Color({ r = 185, g = 125, b = 72, a = 255 })
local woodMid = Color({ r = 155, g = 98, b = 55, a = 255 })
local woodShadow = Color({ r = 112, g = 66, b = 40, a = 255 })
local outlineColor = Color({ r = 72, g = 42, b = 28, a = 255 })
local wornHighlight = Color({ r = 245, g = 214, b = 150, a = 255 })
local mossAccent = Color({ r = 82, g = 128, b = 76, a = 255 })

local LEFT_RAIL_X = 2
local RIGHT_RAIL_X = 11
local RAIL_WIDTH = 3
local RUNG_X = 3
local RUNG_WIDTH = 10

local function drawPixel(img, x, y, color)
	if x >= 0 and x < img.width and y >= 0 and y < img.height then
		img:drawPixel(x, y, color)
	end
end

local function fillRect(img, x, y, w, h, color)
	for py = y, y + h - 1 do
		for px = x, x + w - 1 do
			drawPixel(img, px, py, color)
		end
	end
end

local function fillLine(img, x, y, w, color)
	for px = x, x + w - 1 do
		drawPixel(img, px, y, color)
	end
end

local function drawRail(img, x)
	-- Dark outer edge.
	fillRect(img, x - 1, 0, RAIL_WIDTH + 2, SPRITE_HEIGHT, outlineColor)

	-- Wood body with a top-left style highlight and right-side shadow.
	fillRect(img, x, 1, RAIL_WIDTH, SPRITE_HEIGHT - 2, woodMain)
	fillRect(img, x, 1, 1, SPRITE_HEIGHT - 2, woodHighlight)
	fillRect(img, x + RAIL_WIDTH - 1, 1, 1, SPRITE_HEIGHT - 2, woodShadow)

	-- Subtle plank grain breaks up long configurable heights without becoming noisy.
	for y = 10, SPRITE_HEIGHT - 8, 16 do
		drawPixel(img, x + 1, y, woodMid)
		drawPixel(img, x + 1, y + 1, woodShadow)
	end
end

local function drawRung(img, y, index)
	-- Rungs sit in front of the rails so the player-facing climbable shape reads clearly.
	fillRect(img, RUNG_X, y - 1, RUNG_WIDTH, 4, outlineColor)
	fillLine(img, RUNG_X + 1, y, RUNG_WIDTH - 2, woodHighlight)
	fillLine(img, RUNG_X + 1, y + 1, RUNG_WIDTH - 2, woodMain)
	fillLine(img, RUNG_X + 1, y + 2, RUNG_WIDTH - 2, woodShadow)

	-- Tiny wear marks alternate sides to keep repeated rungs from looking copied.
	if index % 2 == 0 then
		drawPixel(img, RUNG_X + 3, y, wornHighlight)
		drawPixel(img, RUNG_X + RUNG_WIDTH - 3, y + 2, woodMid)
	else
		drawPixel(img, RUNG_X + RUNG_WIDTH - 4, y, wornHighlight)
		drawPixel(img, RUNG_X + 2, y + 2, woodMid)
	end
end

local function drawRailCap(img, x, topY)
	fillRect(img, x - 1, topY, RAIL_WIDTH + 2, 3, outlineColor)
	fillRect(img, x, topY + 1, RAIL_WIDTH, 1, woodHighlight)
	fillRect(img, x, topY + 2, RAIL_WIDTH, 1, woodShadow)
end

local function drawCaps(img)
	drawRailCap(img, LEFT_RAIL_X, 0)
	drawRailCap(img, RIGHT_RAIL_X, 0)
	drawRailCap(img, LEFT_RAIL_X, SPRITE_HEIGHT - 3)
	drawRailCap(img, RIGHT_RAIL_X, SPRITE_HEIGHT - 3)

	-- Small top/bottom braces make the generated full-height preview feel intentional.
	fillRect(img, RUNG_X, 2, RUNG_WIDTH, 3, outlineColor)
	fillLine(img, RUNG_X + 1, 3, RUNG_WIDTH - 2, woodHighlight)
	fillLine(img, RUNG_X + 1, 4, RUNG_WIDTH - 2, woodShadow)

	fillRect(img, RUNG_X, SPRITE_HEIGHT - 5, RUNG_WIDTH, 3, outlineColor)
	fillLine(img, RUNG_X + 1, SPRITE_HEIGHT - 4, RUNG_WIDTH - 2, woodMain)
	fillLine(img, RUNG_X + 1, SPRITE_HEIGHT - 3, RUNG_WIDTH - 2, woodShadow)
end

local function drawMossAccents(img)
	if SPRITE_HEIGHT < 40 then
		return
	end

	-- Sparse green pixels tie the ladder into natural rooms without overpowering the wood.
	drawPixel(img, LEFT_RAIL_X - 1, 7, mossAccent)
	drawPixel(img, LEFT_RAIL_X, 8, mossAccent)
	drawPixel(img, RIGHT_RAIL_X + RAIL_WIDTH, SPRITE_HEIGHT - 10, mossAccent)
	drawPixel(img, RIGHT_RAIL_X + RAIL_WIDTH - 1, SPRITE_HEIGHT - 9, mossAccent)
end

local function drawLadder(img)
	drawRail(img, LEFT_RAIL_X)
	drawRail(img, RIGHT_RAIL_X)
	drawCaps(img)

	local rungIndex = 1
	for y = 10, SPRITE_HEIGHT - 10, RUNG_PITCH do
		drawRung(img, y, rungIndex)
		rungIndex = rungIndex + 1
	end

	drawMossAccents(img)
end

local frame = sprite.frames[1]
frame.duration = 1.0
local cel = sprite.cels[1]
drawLadder(cel.image)

local tag = sprite:newTag(1, 1)
tag.name = "default"
tag.aniDir = AniDir.FORWARD

local outPath = nil
if app.params ~= nil then
	outPath = app.params["out"]
end
if outPath ~= nil and outPath ~= "" then
	sprite:saveAs(outPath)
end

if app.isUIAvailable then
	app.alert("Ladder Sprite Created!")
end
