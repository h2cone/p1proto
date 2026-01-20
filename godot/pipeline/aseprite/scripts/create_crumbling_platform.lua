-- Aseprite Script: Crumbling Platform Generator
-- Creates a 16x8 crumbling platform tile with animations:
--   idle: stable platform
--   shake: shaking with cracks appearing
--   crumble: breaks into falling fragments with particles
-- Width matches player (16px), can be tiled horizontally for longer platforms

local SPRITE_WIDTH = 16
local SPRITE_HEIGHT = 8

-- Extended height for crumble animation (fragments fall below)
local EXTENDED_HEIGHT = 24

-- Create new sprite with extended height for falling animation
local sprite = Sprite(SPRITE_WIDTH, EXTENDED_HEIGHT, ColorMode.RGB)
sprite.filename = "crumbling_platform.aseprite"

-- Define colors - earthy/stone colors to suggest fragility
local topHighlight = Color({ r = 230, g = 200, b = 170, a = 255 }) -- Top edge highlight
local topSurface = Color({ r = 200, g = 170, b = 140, a = 255 }) -- Standing surface
local bodyFill = Color({ r = 170, g = 140, b = 110, a = 255 }) -- Body fill
local bodyEdge = Color({ r = 140, g = 110, b = 85, a = 255 }) -- Body edge/bottom
local crackColor = Color({ r = 80, g = 60, b = 45, a = 255 }) -- Crack lines
local particleLight = Color({ r = 190, g = 160, b = 130, a = 255 }) -- Light particles
local particleDark = Color({ r = 120, g = 90, b = 65, a = 255 }) -- Dark particles

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

-- Draw the stable 16x8 platform tile
local function drawPlatform(img, offsetX, offsetY)
	offsetX = offsetX or 0
	offsetY = offsetY or 0

	-- Row 0: Top highlight
	fillLine(img, offsetX, offsetY + 0, 16, topHighlight)

	-- Row 1: Top surface
	fillLine(img, offsetX, offsetY + 1, 16, topSurface)

	-- Row 2-5: Body fill
	for y = 2, 5 do
		fillLine(img, offsetX, offsetY + y, 16, bodyFill)
	end

	-- Row 6-7: Bottom edge
	fillLine(img, offsetX, offsetY + 6, 16, bodyEdge)
	fillLine(img, offsetX, offsetY + 7, 16, bodyEdge)

	-- Subtle center separation (two 8px segments)
	for y = 2, 6 do
		drawPixel(img, offsetX + 8, offsetY + y, bodyEdge)
	end
end

-- Draw cracks on the 16x8 platform
local function drawCracks(img, intensity, offsetX, offsetY)
	offsetX = offsetX or 0
	offsetY = offsetY or 0

	-- Crack patterns based on intensity (1-3)
	if intensity >= 1 then
		-- First crack: left segment diagonal
		drawPixel(img, offsetX + 3, offsetY + 2, crackColor)
		drawPixel(img, offsetX + 4, offsetY + 3, crackColor)
		drawPixel(img, offsetX + 3, offsetY + 4, crackColor)
	end

	if intensity >= 2 then
		-- Second crack: right segment
		drawPixel(img, offsetX + 11, offsetY + 3, crackColor)
		drawPixel(img, offsetX + 12, offsetY + 4, crackColor)
		drawPixel(img, offsetX + 11, offsetY + 5, crackColor)

		-- Center crack
		drawPixel(img, offsetX + 7, offsetY + 4, crackColor)
		drawPixel(img, offsetX + 8, offsetY + 5, crackColor)
	end

	if intensity >= 3 then
		-- More cracks spreading
		drawPixel(img, offsetX + 1, offsetY + 4, crackColor)
		drawPixel(img, offsetX + 2, offsetY + 5, crackColor)

		drawPixel(img, offsetX + 13, offsetY + 2, crackColor)
		drawPixel(img, offsetX + 14, offsetY + 3, crackColor)

		drawPixel(img, offsetX + 6, offsetY + 2, crackColor)
		drawPixel(img, offsetX + 10, offsetY + 2, crackColor)
	end
end

-- Draw a single fragment (small piece of platform)
local function drawFragment(img, x, y, size, variant)
	local colors = { topSurface, bodyFill, bodyEdge }
	local baseColor = colors[(variant % 3) + 1]

	if size == 1 then
		drawPixel(img, x, y, baseColor)
	elseif size == 2 then
		drawPixel(img, x, y, topSurface)
		drawPixel(img, x + 1, y, baseColor)
		drawPixel(img, x, y + 1, baseColor)
		drawPixel(img, x + 1, y + 1, bodyEdge)
	elseif size == 3 then
		drawPixel(img, x, y, topSurface)
		drawPixel(img, x + 1, y, topSurface)
		drawPixel(img, x + 2, y, baseColor)
		drawPixel(img, x, y + 1, baseColor)
		drawPixel(img, x + 1, y + 1, baseColor)
		drawPixel(img, x + 2, y + 1, bodyEdge)
		drawPixel(img, x + 1, y + 2, bodyEdge)
	end
end

-- Draw particle (dust/debris)
local function drawParticle(img, x, y, variant)
	local color = (variant % 2 == 0) and particleLight or particleDark
	drawPixel(img, x, y, color)
end

-- Fragment data for 16x8 tile: 6 fragments (3 per 8px segment)
local fragments = {
	-- Left segment
	{ dx = 0, dy = 0, size = 3, variant = 0 }, -- top-left chunk
	{ dx = 4, dy = 2, size = 2, variant = 1 }, -- mid-left chunk
	{ dx = 1, dy = 5, size = 2, variant = 2 }, -- bottom-left chunk
	-- Right segment
	{ dx = 9, dy = 0, size = 3, variant = 1 }, -- top-right chunk
	{ dx = 12, dy = 3, size = 2, variant = 0 }, -- mid-right chunk
	{ dx = 10, dy = 5, size = 2, variant = 2 }, -- bottom-right chunk
}

-- Particle spawn positions (relative to center at 8,4)
local particleSpawns = {
	{ dx = -6, dy = 1 },
	{ dx = -3, dy = 0 },
	{ dx = 0, dy = 2 },
	{ dx = 3, dy = 1 },
	{ dx = 6, dy = 0 },
	{ dx = -4, dy = 3 },
	{ dx = 2, dy = 4 },
	{ dx = 5, dy = 3 },
	{ dx = -2, dy = 5 },
}

-- Draw crumbling frame with fragments falling
local function drawCrumbleFrame(img, frameNum, totalFrames)
	local centerX = 8
	local startY = 0

	-- Calculate fall progress (0.0 to 1.0)
	local progress = frameNum / totalFrames

	-- Draw fragments
	for i, frag in ipairs(fragments) do
		-- Each fragment has slightly different timing
		local fragDelay = (i - 1) * 0.04
		local fragProgress = math.max(0, progress - fragDelay)

		if fragProgress > 0 then
			-- Horizontal spread: fragments move outward from center
			local spreadDir = (frag.dx < centerX) and -1 or 1
			local spreadX = spreadDir * fragProgress * (2 + i * 0.4)

			-- Vertical fall with gravity
			local fallY = fragProgress * fragProgress * 18

			-- Final position
			local finalX = frag.dx + math.floor(spreadX)
			local finalY = startY + frag.dy + math.floor(fallY)

			-- Only draw if still in view and not faded out
			if finalY < EXTENDED_HEIGHT - 3 and fragProgress < 0.85 then
				drawFragment(img, finalX, finalY, frag.size, frag.variant)
			end
		end
	end

	-- Draw particles
	for i, p in ipairs(particleSpawns) do
		local particleDelay = 0.08 + (i * 0.05)
		local particleProgress = math.max(0, progress - particleDelay)

		if particleProgress > 0 and particleProgress < 0.65 then
			-- Particles fall faster and scatter more
			local scatterX = p.dx + math.floor(particleProgress * p.dx * 0.6)
			local fallY = p.dy + math.floor(particleProgress * particleProgress * 22)

			local finalX = centerX + scatterX
			local finalY = startY + fallY

			if finalX >= 0 and finalX < img.width and finalY < EXTENDED_HEIGHT - 1 then
				drawParticle(img, finalX, finalY, i)
			end
		end
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

-- ==================== IDLE (Static) ====================
local idleStart = frameIndex
local frame = newFrame()
frame.duration = 1.0
local cel = newCelWithImage(frame)

drawPlatform(cel.image, 0, 0)

defineTag("idle", idleStart, idleStart)

-- ==================== SHAKE (Shaking with cracks) ====================
local shakeStart = frameIndex

-- Shake animation: 6 frames with increasing cracks
local shakeOffsets = {
	{ x = 0, y = 0, crack = 0 }, -- Frame 1: no shake yet
	{ x = -1, y = 0, crack = 1 }, -- Frame 2: shake left, first cracks
	{ x = 1, y = 0, crack = 1 }, -- Frame 3: shake right
	{ x = -1, y = 0, crack = 2 }, -- Frame 4: shake left, more cracks
	{ x = 1, y = 0, crack = 2 }, -- Frame 5: shake right
	{ x = 0, y = 0, crack = 3 }, -- Frame 6: settle, max cracks
}

for i, shake in ipairs(shakeOffsets) do
	local f = newFrame()
	f.duration = 0.08 -- Fast shake
	local c = newCelWithImage(f)

	drawPlatform(c.image, shake.x, shake.y)
	if shake.crack > 0 then
		drawCracks(c.image, shake.crack, shake.x, shake.y)
	end
end

local shakeEnd = frameIndex - 1
defineTag("shake", shakeStart, shakeEnd)

-- ==================== CRUMBLE (Breaking apart) ====================
local crumbleStart = frameIndex

-- Crumble animation: 10 frames of fragments falling
local crumbleFrames = 10

for i = 1, crumbleFrames do
	local f = newFrame()
	f.duration = 0.07 -- Slightly faster for dramatic effect
	local c = newCelWithImage(f)

	drawCrumbleFrame(c.image, i, crumbleFrames)
end

local crumbleEnd = frameIndex - 1
defineTag("crumble", crumbleStart, crumbleEnd)

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
	app.alert("Crumbling Platform Sprite Created!")
end
