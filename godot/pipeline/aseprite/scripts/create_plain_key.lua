-- Aseprite Script: Key Sprite Generator
-- Creates a 16x16 key with spinning animation

local sprite = Sprite(16, 16, ColorMode.RGB)
sprite.filename = "plain_key.aseprite"

-- Palette
local goldLight = Color({ r = 255, g = 223, b = 0, a = 255 })
local goldShadow = Color({ r = 184, g = 134, b = 11, a = 255 })
local highlight = Color({ r = 255, g = 255, b = 224, a = 255 })
local black = Color({ r = 0, g = 0, b = 0, a = 255 })

-- Params
local centerX = 8
local headY = 2
local headRadius = 3
local shaftTop = 5
local shaftBottom = 13
local teethY = 11
local teethSize = 2

local numFrames = 8

local function drawRect(img, x, y, w, h, color)
	for py = y, y + h - 1 do
		for px = x, x + w - 1 do
			if px >= 0 and px < img.width and py >= 0 and py < img.height then
				img:drawPixel(px, py, color)
			end
		end
	end
end

-- Function to draw key at a specific rotation angle (0 to 2pi)
local function drawKey(cel, angle)
	local img = cel.image
	local scale = math.cos(angle) -- -1 to 1

	-- When scale is near 0, the key is thin (edge view)
	local widthScale = math.abs(scale)
	-- Avoid 0 width
	if widthScale < 0.1 then
		widthScale = 0.1
	end

	-- 1. Draw Head (Circle-ish)
	-- We squash the circle horizontally based on scale
	local currentHeadW = math.max(1, math.floor(headRadius * 2 * widthScale))
	local currentHeadX = centerX - math.floor(currentHeadW / 2)

	-- Draw Head Ring
	drawRect(img, currentHeadX, headY, currentHeadW, headRadius * 2, goldShadow)
	-- Inner hole (smaller)
	if widthScale > 0.4 then
		local innerW = math.max(1, currentHeadW - 2)
		local innerX = centerX - math.floor(innerW / 2)
		drawRect(img, innerX, headY + 1, innerW, headRadius * 2 - 2, black) -- transparency hole? actually we just clear or draw background color?
		-- Aseprite ColorMode.RGB transparency is usually mask color or alpha=0.
		-- We'll just draw the ring by drawing the full shape then clearing center if we want hole.
		-- Simpler: Just draw a solid head for pixel art at this scale, maybe a dot in middle.
		drawRect(img, innerX, headY + 1, innerW, headRadius * 2 - 2, Color({ r = 0, 0, 0, a = 0 })) -- Clear
	end

	-- 2. Draw Shaft
	-- Shaft is central, thickness doesn't change much, maybe 1px or 2px
	local shaftW = (widthScale < 0.3) and 1 or 2
	local shaftX = centerX - math.floor(shaftW / 2)
	drawRect(img, shaftX, shaftTop, shaftW, shaftBottom - shaftTop, goldLight)

	-- 3. Draw Teeth
	-- Teeth move based on scale.
	-- If scale > 0, teeth on Right. If scale < 0, teeth on Left.
	local teethOffsetMax = 3
	local teethCurrentOffset = math.floor(teethOffsetMax * scale)

	-- Teeth are attached to shaft.
	local teethX
	if scale >= 0 then
		teethX = centerX + 1
		local tW = teethCurrentOffset
		if tW > 0 then
			drawRect(img, teethX, teethY, tW, 1, goldLight)
			drawRect(img, teethX, teethY + 2, tW, 1, goldLight)
		end
	else
		-- Left side
		local tW = math.abs(teethCurrentOffset)
		teethX = centerX - 1 - tW
		if tW > 0 then
			drawRect(img, teethX, teethY, tW, 1, goldLight)
			drawRect(img, teethX, teethY + 2, tW, 1, goldLight)
		end
	end
end

-- Generate Frames
local frameIndex = 1
local tagStart = 1

for i = 0, numFrames - 1 do
	local frame
	if i == 0 then
		frame = sprite.frames[1]
	else
		frame = sprite:newEmptyFrame(frameIndex)
	end
	frame.duration = 0.1

	local cel
	if i == 0 then
		cel = sprite.cels[1]
	else
		cel = sprite:newCel(sprite.layers[1], frame)
	end

	-- Angle: 0 to 2pi
	local angle = (i / numFrames) * 2 * math.pi
	drawKey(cel, angle)

	frameIndex = frameIndex + 1
end

local tag = sprite:newTag(tagStart, numFrames)
tag.name = "spin"
tag.aniDir = AniDir.FORWARD

app.alert("Key Sprite Created!")
