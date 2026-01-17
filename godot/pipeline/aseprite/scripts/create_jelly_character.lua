-- Aseprite Script: Jelly Character Sprite Sheet Generator
-- Creates a 16x24 cyan jelly character with cute animations

-- Create new sprite
local sprite = Sprite(16, 24, ColorMode.RGB)
sprite.filename = "jelly_character.aseprite"

-- Remove the default frame that Aseprite creates
sprite:deleteFrame(1)

-- Define colors
local cyan = Color({ r = 0, g = 255, b = 255, a = 255 })
local darkCyan = Color({ r = 0, g = 180, b = 180, a = 255 })
local white = Color({ r = 255, g = 255, b = 255, a = 255 })
local black = Color({ r = 0, g = 0, b = 0, a = 255 })

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

-- Helper function to draw eyes (facing right by default)
local function drawEyes(cel, offsetX, offsetY, pupilOffsetX, pupilOffsetY, facingRight)
	local img = cel.image
	local eyeX = facingRight and 10 or 3
	local eye1X = eyeX + offsetX + pupilOffsetX
	local eye2X = eyeX + 3 + offsetX + pupilOffsetX
	local eyeY = 8 + offsetY + pupilOffsetY

	-- White of eyes
	img:drawPixel(eye1X, eyeY, white)
	img:drawPixel(eye2X, eyeY, white)
	img:drawPixel(eye1X, eyeY + 1, white)
	img:drawPixel(eye2X, eyeY + 1, white)

	-- Pupils
	local pupilY = eyeY + 1
	img:drawPixel(eye1X, pupilY, black)
	img:drawPixel(eye2X, pupilY, black)
end

-- Helper function to draw jelly body with squash/stretch
local function drawJellyBody(cel, x, y, width, height, facingRight)
	-- Main body
	fillRect(cel, x, y, width, height, cyan)

	-- Add darker outline for depth
	local img = cel.image
	-- Top edge
	for px = x, x + width - 1 do
		img:drawPixel(px, y, darkCyan)
	end
	-- Bottom edge
	for px = x, x + width - 1 do
		img:drawPixel(px, y + height - 1, darkCyan)
	end
	-- Left edge
	for py = y, y + height - 1 do
		img:drawPixel(x, py, darkCyan)
	end
	-- Right edge
	for py = y, y + height - 1 do
		img:drawPixel(x + width - 1, py, darkCyan)
	end
end

-- Store tag info to create them after all frames exist
local tagInfos = {}

-- Create tag helper - stores info for later creation
local function queueTag(name, fromFrame, toFrame)
	table.insert(tagInfos, { name = name, fromFrame = fromFrame, toFrame = toFrame })
end

-- Create all tags after all frames are created
local function createAllTags()
	for _, info in ipairs(tagInfos) do
		local tag = sprite:newTag(info.fromFrame, info.toFrame)
		tag.name = info.name
	end
end

local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1

-- ==================== IDLE ANIMATION ====================
-- 4 frames, gentle breathing effect
local idleStart = frameIndex
for i = 1, 4 do
	local frame = sprite:newEmptyFrame(frameIndex)
	frameIndex = frameIndex + 1
	frame.duration = 0.15

	local cel = newCelWithImage(frame)

	-- Gentle vertical bob
	local yOffset = 0
	if i == 1 or i == 4 then
		yOffset = 0
	elseif i == 2 then
		yOffset = -1
	else
		yOffset = 0
	end

	-- Slight width variation for breathing
	local widthVar = (i == 2 or i == 3) and 1 or 0
	local xOffset = widthVar > 0 and -1 or 0

	drawJellyBody(cel, xOffset, yOffset, 16 + widthVar, 24 - yOffset, true)
	drawEyes(cel, xOffset, yOffset, 0, 0, true)
end
queueTag("idle", idleStart, frameIndex - 1)

-- ==================== WALK ANIMATION ====================
-- 6 frames, bouncy walk cycle
local walkStart = frameIndex
for i = 1, 6 do
	local frame = sprite:newEmptyFrame(frameIndex)
	frameIndex = frameIndex + 1
	frame.duration = 0.1

	local cel = newCelWithImage(frame)

	-- Squash and stretch for bouncy walk
	local yOffset, heightMod, widthMod, xMod = 0, 0, 0, 0

	if i == 1 or i == 4 then
		-- Compressed (landing)
		yOffset = 2
		heightMod = -3
		widthMod = 2
		xMod = -1
	elseif i == 2 or i == 5 then
		-- Normal
		yOffset = 0
		heightMod = 0
		widthMod = 0
		xMod = 0
	else
		-- Stretched (pushing up)
		yOffset = -1
		heightMod = 2
		widthMod = -1
		xMod = 1
	end

	-- Eye blink on compressed frames
	local eyeOffsetY = (i == 1 or i == 4) and 1 or 0

	drawJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, true)
	drawEyes(cel, xMod, yOffset, 0, eyeOffsetY, true)
end
queueTag("walk", walkStart, frameIndex - 1)

-- ==================== JUMP ANIMATION ====================
-- 3 frames, anticipation and launch
local jumpStart = frameIndex
for i = 1, 3 do
	local frame = sprite:newEmptyFrame(frameIndex)
	frameIndex = frameIndex + 1
	frame.duration = 0.08

	local cel = newCelWithImage(frame)

	local yOffset, heightMod, widthMod, xMod = 0, 0, 0, 0

	if i == 1 then
		-- Squash down (anticipation)
		yOffset = 4
		heightMod = -6
		widthMod = 3
		xMod = -1
	elseif i == 2 then
		-- Starting to extend
		yOffset = 1
		heightMod = 0
		widthMod = 1
		xMod = -1
	else
		-- Full stretch upward
		yOffset = -2
		heightMod = 4
		widthMod = -2
		xMod = 1
	end

	drawJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, true)
	drawEyes(cel, xMod, yOffset, 0, -1, true)
end
queueTag("jump", jumpStart, frameIndex - 1)

-- ==================== FALL ANIMATION ====================
-- 2 frames, stretched in air
local fallStart = frameIndex
for i = 1, 2 do
	local frame = sprite:newEmptyFrame(frameIndex)
	frameIndex = frameIndex + 1
	frame.duration = 0.15

	local cel = newCelWithImage(frame)

	-- Stretched vertically, slight wobble
	local yOffset = -1
	local heightMod = 3
	local widthMod = (i == 1) and -1 or -2
	local xMod = (i == 1) and 0 or 1

	drawJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, true)
	drawEyes(cel, xMod, yOffset, 0, 1, true)
end
queueTag("fall", fallStart, frameIndex - 1)

-- ==================== LAND ANIMATION ====================
-- 4 frames, bouncy landing with recovery
local landStart = frameIndex
for i = 1, 4 do
	local frame = sprite:newEmptyFrame(frameIndex)
	frameIndex = frameIndex + 1
	frame.duration = 0.08

	local cel = newCelWithImage(frame)

	local yOffset, heightMod, widthMod, xMod = 0, 0, 0, 0

	if i == 1 then
		-- Maximum squash
		yOffset = 5
		heightMod = -8
		widthMod = 4
		xMod = -2
	elseif i == 2 then
		-- Bounce back over-stretch
		yOffset = -2
		heightMod = 3
		widthMod = -1
		xMod = 1
	elseif i == 3 then
		-- Small squash
		yOffset = 2
		heightMod = -3
		widthMod = 2
		xMod = -1
	else
		-- Return to normal
		yOffset = 0
		heightMod = 0
		widthMod = 0
		xMod = 0
	end

	drawJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, true)
	drawEyes(cel, xMod, yOffset, 0, 0, true)
end
queueTag("land", landStart, frameIndex - 1)

-- ==================== DEATH ANIMATION ====================
-- 6 frames, shock then splat/dissolve effect
local deathStart = frameIndex

-- Define red/pink colors for hurt effect
local hurtPink = Color({ r = 255, g = 100, b = 150, a = 255 })
local hurtRed = Color({ r = 255, g = 50, b = 100, a = 255 })

-- Helper function to draw hurt jelly body (pink/red tint)
local function drawHurtJellyBody(cel, x, y, width, height, alpha)
	local mainColor = Color({ r = hurtPink.red, g = hurtPink.green, b = hurtPink.blue, a = alpha })
	local outlineColor = Color({ r = hurtRed.red, g = hurtRed.green, b = hurtRed.blue, a = alpha })

	-- Main body
	local img = cel.image
	for py = y, y + height - 1 do
		for px = x, x + width - 1 do
			if px >= 0 and px < img.width and py >= 0 and py < img.height then
				img:drawPixel(px, py, mainColor)
			end
		end
	end

	-- Outline
	for px = x, x + width - 1 do
		if px >= 0 and px < img.width and y >= 0 and y < img.height then
			img:drawPixel(px, y, outlineColor)
		end
		if px >= 0 and px < img.width and y + height - 1 >= 0 and y + height - 1 < img.height then
			img:drawPixel(px, y + height - 1, outlineColor)
		end
	end
	for py = y, y + height - 1 do
		if x >= 0 and x < img.width and py >= 0 and py < img.height then
			img:drawPixel(x, py, outlineColor)
		end
		if x + width - 1 >= 0 and x + width - 1 < img.width and py >= 0 and py < img.height then
			img:drawPixel(x + width - 1, py, outlineColor)
		end
	end
end

-- Helper function to draw X eyes (dead eyes)
local function drawDeadEyes(cel, offsetX, offsetY, alpha)
	local img = cel.image
	local eyeColor = Color({ r = 0, g = 0, b = 0, a = alpha })
	local eyeX = 10 + offsetX
	local eyeY = 8 + offsetY

	-- X pattern for first eye
	img:drawPixel(eyeX, eyeY, eyeColor)
	img:drawPixel(eyeX + 1, eyeY + 1, eyeColor)
	img:drawPixel(eyeX + 1, eyeY, eyeColor)
	img:drawPixel(eyeX, eyeY + 1, eyeColor)

	-- X pattern for second eye
	img:drawPixel(eyeX + 3, eyeY, eyeColor)
	img:drawPixel(eyeX + 4, eyeY + 1, eyeColor)
	img:drawPixel(eyeX + 4, eyeY, eyeColor)
	img:drawPixel(eyeX + 3, eyeY + 1, eyeColor)
end

for i = 1, 6 do
	local frame = sprite:newEmptyFrame(frameIndex)
	frameIndex = frameIndex + 1
	frame.duration = (i <= 2) and 0.08 or 0.12

	local cel = newCelWithImage(frame)

	local yOffset, heightMod, widthMod, xMod = 0, 0, 0, 0
	local alpha = 255

	if i == 1 then
		-- Initial shock - slight stretch upward
		yOffset = -2
		heightMod = 2
		widthMod = -1
		xMod = 0
		drawHurtJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, alpha)
		drawEyes(cel, xMod, yOffset, 0, -1, true)
	elseif i == 2 then
		-- Shake/vibrate frame
		yOffset = -1
		heightMod = 1
		widthMod = 0
		xMod = 1
		drawHurtJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, alpha)
		drawDeadEyes(cel, xMod, yOffset, alpha)
	elseif i == 3 then
		-- Start squashing down
		yOffset = 3
		heightMod = -5
		widthMod = 3
		xMod = -1
		drawHurtJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, alpha)
		drawDeadEyes(cel, xMod, yOffset, alpha)
	elseif i == 4 then
		-- More squashed, starting to fade
		yOffset = 6
		heightMod = -10
		widthMod = 5
		xMod = -2
		alpha = 200
		drawHurtJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, alpha)
		drawDeadEyes(cel, xMod, yOffset, alpha)
	elseif i == 5 then
		-- Almost flat, more faded
		yOffset = 10
		heightMod = -16
		widthMod = 6
		xMod = -3
		alpha = 130
		drawHurtJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, alpha)
	else
		-- Final splat - nearly invisible puddle
		yOffset = 14
		heightMod = -20
		widthMod = 8
		xMod = -4
		alpha = 60
		drawHurtJellyBody(cel, xMod, yOffset, 16 + widthMod, 24 + heightMod, alpha)
	end
end
queueTag("death", deathStart, frameIndex - 1)

-- Create all tags now that all frames exist
createAllTags()

-- Set to loop all animations
for i, tag in ipairs(sprite.tags) do
	tag.aniDir = AniDir.FORWARD
end

app.alert("Jelly Character Sprite Sheet Created!")
