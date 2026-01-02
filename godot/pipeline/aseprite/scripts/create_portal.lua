-- Aseprite Script: Portal Sprite Generator
-- Creates a 32x48 magical portal (visually harmonious with the cyan jelly character)

-- Create new sprite (32x48 is a multiple of 8 and fits the 16x24 player well)
local sprite = Sprite(32, 48, ColorMode.RGB)
sprite.filename = "portal.aseprite"

-- Define colors (Purple/Pink theme to contrast with Cyan player but keep the neon vibe)
local brightPurple = Color({ r = 255, g = 100, b = 255, a = 255 })
local mainPurple = Color({ r = 180, g = 0, b = 220, a = 255 })
local darkPurple = Color({ r = 80, g = 0, b = 120, a = 255 })
local white = Color({ r = 255, g = 255, b = 255, a = 255 })
local magicGlow = Color({ r = 100, g = 200, b = 255, a = 150 }) -- Semi-transparent cyan glow

-- Helper function to draw a filled oval (rough approximation)
local function drawOval(img, cx, cy, rx, ry, color)
	for y = cy - ry, cy + ry do
		for x = cx - rx, cx + rx do
			if ((x - cx) ^ 2 / rx ^ 2) + ((y - cy) ^ 2 / ry ^ 2) <= 1.0 then
				if x >= 0 and x < img.width and y >= 0 and y < img.height then
					img:drawPixel(x, y, color)
				end
			end
		end
	end
end

-- Helper function to draw the portal frame
local function drawPortalFrame(img, cx, cy, rx, ry)
	-- Outer dark rim
	drawOval(img, cx, cy, rx, ry, darkPurple)
	-- Inner main rim
	drawOval(img, cx, cy, rx - 2, ry - 2, mainPurple)
	-- Center "hole" (cleared for the vortex)
	drawOval(img, cx, cy, rx - 4, ry - 4, Color({ r = 0, g = 0, b = 0, a = 0 }))
end

-- Helper function to draw vortex particles
local function drawVortex(img, cx, cy, frameNum, maxFrames)
	local radius = 10
	local particles = 8
	local angleOffset = (frameNum / maxFrames) * (2 * math.pi)

	for i = 0, particles - 1 do
		local angle = (i / particles) * (2 * math.pi) + angleOffset

		-- Spiral effect
		local r = radius * (0.5 + 0.5 * math.sin(angle * 2 + angleOffset))

		local px = cx + math.cos(angle) * r
		local py = cy + math.sin(angle) * (r * 1.5) -- Stretch vertically

		-- Quantize to pixels
		px = math.floor(px + 0.5)
		py = math.floor(py + 0.5)

		if px >= 0 and px < img.width and py >= 0 and py < img.height then
			img:drawPixel(px, py, brightPurple)
			img:drawPixel(px + 1, py, brightPurple) -- Make particle slightly bigger
		end
	end

	-- Center core
	img:drawPixel(cx, cy, white)
	img:drawPixel(cx + 1, cy, white)
	img:drawPixel(cx, cy + 1, white)
	img:drawPixel(cx + 1, cy + 1, white)
end

local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1

-- ==================== IDLE ANIMATION ====================
-- 8 frames loop
local idleStart = frameIndex
local frameCount = 8

for i = 1, frameCount do
	local frame = (i == 1) and sprite.frames[1] or sprite:newEmptyFrame(frameIndex)
	if i > 1 then
		frameIndex = frameIndex + 1
	end
	frame.duration = 0.1

	local cel = newCelWithImage(frame)
	local img = cel.image

	-- Center coordinates
	local cx = 16
	local cy = 24

	-- Pulse effect for the frame
	local pulse = math.sin((i / frameCount) * 2 * math.pi) * 1
	local rx = 14 + pulse
	local ry = 20 + pulse

	drawPortalFrame(img, cx, cy, rx, ry)

	-- Draw background of the vortex (deep void)
	drawOval(img, cx, cy, rx - 4, ry - 4, Color({ r = 20, g = 0, b = 40, a = 255 }))

	-- Draw swirling vortex
	drawVortex(img, cx, cy, i, frameCount)
end

-- Create tag
sprite:newTag(idleStart, frameIndex)
sprite.tags[#sprite.tags].name = "idle"

-- Set to loop
for i, tag in ipairs(sprite.tags) do
	tag.aniDir = AniDir.FORWARD
end

app.alert("Portal Sprite Sheet Created!")
