-- Aseprite Script: Pixel Water Generator
-- Creates one source sprite with tagged water animations for WaterZone.

local WIDTH = 48
local HEIGHT = 32

local sprite = Sprite(WIDTH, HEIGHT, ColorMode.RGB)
sprite.filename = "water.aseprite"

local transparent = Color({ r = 0, g = 0, b = 0, a = 0 })
local shadow = Color({ r = 7, g = 32, b = 54, a = 255 })
local deep = Color({ r = 13, g = 72, b = 111, a = 255 })
local mid = Color({ r = 20, g = 102, b = 146, a = 255 })
local crest = Color({ r = 48, g = 177, b = 204, a = 255 })
local glint = Color({ r = 132, g = 238, b = 247, a = 255 })
local pale = Color({ r = 190, g = 250, b = 255, a = 255 })

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
		drawPixel(img, x + px - x, y, color)
	end
end

local function clear(img)
	fillRect(img, 0, 0, WIDTH, HEIGHT, transparent)
end

local function newCel(frame)
	local img = Image(WIDTH, HEIGHT)
	clear(img)
	return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1
local tags = {}

local function newFrame(duration)
	local frame
	if frameIndex == 1 then
		frame = sprite.frames[1]
	else
		frame = sprite:newEmptyFrame()
	end
	frame.duration = duration
	frameIndex = frameIndex + 1
	return frame
end

local function tag(name, fromFrame, toFrame)
	table.insert(tags, { name = name, fromFrame = fromFrame, toFrame = toFrame })
end

local function drawSegment(img, x, y, w, color)
	fillRect(img, x, y, w, 1, color)
end

local function drawSurface(img, phase)
	fillRect(img, 0, 0, WIDTH, HEIGHT, deep)
	fillRect(img, 0, 0, WIDTH, 1, shadow)
	fillRect(img, 0, 1, WIDTH, 2, mid)

	local drift = phase % 4
	drawSegment(img, 1 + drift, 1, 7, glint)
	drawSegment(img, 13 + drift, 1, 5, crest)
	drawSegment(img, 24 + drift, 1, 8, glint)
	drawSegment(img, 39 + drift, 1, 6, crest)

	drawSegment(img, 7 + drift, 3, 6, crest)
	drawSegment(img, 20 + drift, 3, 4, mid)
	drawSegment(img, 33 + drift, 3, 5, crest)
	drawSegment(img, 45 + drift - WIDTH, 3, 5, crest)

	drawSegment(img, 4 + drift, 5, 5, shadow)
	drawSegment(img, 28 + drift, 5, 6, shadow)

	drawPixel(img, 11 + (phase % 3), 11, glint)
	drawPixel(img, 29 + (phase % 4), 16, crest)
	drawPixel(img, 43 - (phase % 3), 23, mid)
end

local function drawFill(img, phase)
	fillRect(img, 0, 0, WIDTH, HEIGHT, deep)

	local drift = phase % 2
	drawSegment(img, 6 + drift, 8, 5, mid)
	drawSegment(img, 31 - drift, 12, 4, mid)
	drawSegment(img, 17 + drift, 22, 6, mid)
	drawSegment(img, 39 - drift, 28, 4, shadow)

	drawPixel(img, 13 + drift, 17, glint)
	drawPixel(img, 34 - drift, 25, crest)
end

local function drawRipple(img, centerX, baseY, radius, color)
	for dx = -radius, radius do
		if math.abs(dx) == radius or dx % 5 == 0 then
			drawPixel(img, centerX + dx, baseY, color)
		end
	end
end

local function drawDroplet(img, x, y, color)
	drawPixel(img, x, y, color)
end

local function drawEnterSplash(img, phase)
	local cx = 24
	local y = 17
	drawRipple(img, cx, y + math.floor(phase / 2), 5 + phase, crest)
	if phase < 4 then
		drawDroplet(img, cx, y - 5 - phase, glint)
	end
	if phase == 1 or phase == 2 then
		drawDroplet(img, cx - 6 - phase, y - 1 - phase, crest)
		drawDroplet(img, cx + 6 + phase, y - 1 - phase, crest)
	end
end

local function drawExitSplash(img, phase)
	local cx = 24
	local y = 17
	drawRipple(img, cx, y + math.floor(phase / 2), 4 + phase, crest)
	if phase < 3 then
		drawDroplet(img, cx - 4, y - 3 - phase, glint)
		drawDroplet(img, cx + 5, y - 2 - phase, crest)
	end
end

local function drawBubbles(img, phase, strong)
	local baseY = strong and 21 or 19
	local count = strong and 4 or 2
	for i = 1, count do
		local x = 10 + i * 7 + ((phase + i) % 2)
		local y = baseY - phase * 2 - (i % 4)
		drawPixel(img, x, y, pale)
	end
	if strong then
		drawRipple(img, 24, 24, 4 + phase, crest)
	end
end

local start = frameIndex
for i = 0, 3 do
	local frame = newFrame(0.12)
	local cel = newCel(frame)
	drawSurface(cel.image, i * 4)
end
tag("surface_loop", start, frameIndex - 1)

start = frameIndex
for i = 0, 1 do
	local frame = newFrame(0.2)
	local cel = newCel(frame)
	drawFill(cel.image, i * 5)
end
tag("fill_loop", start, frameIndex - 1)

start = frameIndex
for i = 0, 5 do
	local frame = newFrame(0.07)
	local cel = newCel(frame)
	drawEnterSplash(cel.image, i)
end
tag("enter_splash", start, frameIndex - 1)

start = frameIndex
for i = 0, 3 do
	local frame = newFrame(0.08)
	local cel = newCel(frame)
	drawExitSplash(cel.image, i)
end
tag("exit_splash", start, frameIndex - 1)

start = frameIndex
for i = 0, 3 do
	local frame = newFrame(0.09)
	local cel = newCel(frame)
	drawBubbles(cel.image, i, true)
end
tag("dive_bubbles", start, frameIndex - 1)

start = frameIndex
for i = 0, 3 do
	local frame = newFrame(0.11)
	local cel = newCel(frame)
	drawBubbles(cel.image, i, false)
end
tag("swim_bubbles", start, frameIndex - 1)

for _, t in ipairs(tags) do
	local aseTag = sprite:newTag(t.fromFrame, t.toFrame)
	aseTag.name = t.name
	aseTag.aniDir = AniDir.FORWARD
end

local outPath = nil
if app.params ~= nil then
	outPath = app.params["out"]
end
if outPath ~= nil and outPath ~= "" then
	print("Saving pixel water sprite to " .. outPath)
	sprite:saveAs(outPath)
end

if app.isUIAvailable then
	app.alert("Pixel Water Sprite Created!")
end
