-- Aseprite Script: Star Collectible Generator
-- Creates a star with shimmering idle animation and small UI version

local SPRITE_WIDTH = 16
local SPRITE_HEIGHT = 16

-- Create new sprite
local sprite = Sprite(SPRITE_WIDTH, SPRITE_HEIGHT, ColorMode.RGB)
sprite.filename = "collectible_star.aseprite"

-- Define colors - golden star palette
local yellow = Color({ r = 255, g = 230, b = 50, a = 255 })
local gold = Color({ r = 255, g = 200, b = 0, a = 255 })
local orange = Color({ r = 255, g = 160, b = 0, a = 255 })
local white = Color({ r = 255, g = 255, b = 255, a = 255 })
local lightYellow = Color({ r = 255, g = 250, b = 200, a = 255 })
local darkGold = Color({ r = 200, g = 140, b = 0, a = 255 })

-- Sparkle colors for shimmer effect
local sparkleWhite = Color({ r = 255, g = 255, b = 255, a = 255 })
local sparkleYellow = Color({ r = 255, g = 255, b = 180, a = 255 })

-- Helper function to draw a pixel safely
local function drawPixel(img, x, y, color)
	if x >= 0 and x < img.width and y >= 0 and y < img.height then
		img:drawPixel(x, y, color)
	end
end

-- Draw a 5-pointed star shape (16x16)
local function drawStar(img, centerX, centerY, outerR, innerR, baseColor, highlightColor, shadowColor)
	-- Star points coordinates (5-pointed star)
	local points = {}
	local numPoints = 5

	for i = 0, numPoints * 2 - 1 do
		local angle = (i * math.pi / numPoints) - math.pi / 2
		local r = (i % 2 == 0) and outerR or innerR
		local px = centerX + r * math.cos(angle)
		local py = centerY + r * math.sin(angle)
		table.insert(points, { x = px, y = py })
	end

	-- Fill star using scanline approach
	local minY = math.floor(centerY - outerR)
	local maxY = math.ceil(centerY + outerR)

	for y = minY, maxY do
		local intersections = {}
		local n = #points
		for i = 1, n do
			local p1 = points[i]
			local p2 = points[(i % n) + 1]
			if (p1.y <= y and p2.y > y) or (p2.y <= y and p1.y > y) then
				local t = (y - p1.y) / (p2.y - p1.y)
				local x = p1.x + t * (p2.x - p1.x)
				table.insert(intersections, x)
			end
		end
		table.sort(intersections)
		for i = 1, #intersections - 1, 2 do
			local x1 = math.floor(intersections[i])
			local x2 = math.ceil(intersections[i + 1])
			for x = x1, x2 do
				-- Shading: left/top = highlight, right/bottom = shadow
				local color = baseColor
				local distFromCenter = math.sqrt((x - centerX) ^ 2 + (y - centerY) ^ 2)
				if x < centerX - 1 and y < centerY then
					color = highlightColor
				elseif x > centerX + 1 or y > centerY + 1 then
					color = shadowColor
				end
				drawPixel(img, x, y, color)
			end
		end
	end
end

-- Draw sparkle/shimmer effect at specific positions
local function drawSparkle(img, x, y, intensity)
	if intensity == 0 then
		return
	end

	local color = (intensity == 2) and sparkleWhite or sparkleYellow

	-- Cross pattern sparkle
	drawPixel(img, x, y, color)
	if intensity == 2 then
		drawPixel(img, x - 1, y, sparkleYellow)
		drawPixel(img, x + 1, y, sparkleYellow)
		drawPixel(img, x, y - 1, sparkleYellow)
		drawPixel(img, x, y + 1, sparkleYellow)
	end
end

-- Helper to create new cel with image
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

-- Sparkle positions that cycle through the animation
local sparklePositions = {
	{ x = 2, y = 3 },
	{ x = 13, y = 2 },
	{ x = 14, y = 10 },
	{ x = 1, y = 11 },
	{ x = 7, y = 1 },
	{ x = 8, y = 14 },
}

-- ==================== IDLE (Shimmering) ====================
-- 8 frames of shimmering animation
local idleStart = frameIndex
local numIdleFrames = 8

for i = 0, numIdleFrames - 1 do
	local frame = newFrame()
	frame.duration = 0.12

	local cel = newCelWithImage(frame)
	local img = cel.image

	-- Draw the main star
	drawStar(img, 8, 8, 7, 3, gold, lightYellow, orange)

	-- Add cycling sparkles for shimmer effect
	for j, pos in ipairs(sparklePositions) do
		-- Each sparkle has a different phase
		local phase = (i + j * 2) % numIdleFrames
		local intensity = 0
		if phase == 0 then
			intensity = 2 -- Bright
		elseif phase == 1 or phase == 7 then
			intensity = 1 -- Dim
		end
		drawSparkle(img, pos.x, pos.y, intensity)
	end
end

defineTag("idle", idleStart, frameIndex - 1)

-- Create all tags
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
	app.alert("Star Collectible Sprite Created!")
end
