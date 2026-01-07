-- Aseprite Script: Switch Door Generator
-- Creates a 16x48 mechanical door (multiple of 8 height)

local sprite = Sprite(16, 48, ColorMode.RGB)
sprite.filename = "switch_door.aseprite"

-- Palette
local metalLight = Color({ r = 200, g = 200, b = 200, a = 255 })
local metalBase = Color({ r = 169, g = 169, b = 169, a = 255 })
local metalDark = Color({ r = 100, g = 100, b = 100, a = 255 })
local black = Color({ r = 0, g = 0, b = 0, a = 255 })
local lightRed = Color({ r = 255, g = 50, b = 50, a = 255 })
local lightGreen = Color({ r = 50, g = 255, b = 50, a = 255 })

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

-- Helper: Draw Door Frame (Static part)
local function drawFrame(img)
	-- Top housing (4 pixels high)
	fillRect(img, 0, 0, 16, 4, metalDark)
	fillRect(img, 1, 1, 14, 2, metalBase)
end

-- Helper: Draw Door Panel (Moving part)
local function drawPanel(img, panelHeight, lightColor)
	if panelHeight <= 0 then
		return
	end

	local endY = 4 + panelHeight

	-- Main panel body
	fillRect(img, 2, 4, 12, panelHeight, metalBase)

	-- Outline/Detail
	fillRect(img, 2, 4, 1, panelHeight, metalLight) -- Left highlight
	fillRect(img, 13, 4, 1, panelHeight, metalDark) -- Right shadow
	fillRect(img, 2, endY - 1, 12, 1, metalDark) -- Bottom edge

	-- Horizontal bars
	for y = 4 + 8, endY - 2, 8 do
		fillRect(img, 3, y, 10, 1, metalDark)
	end

	-- Center Light
	local lightY = 24
	if endY > lightY + 4 then
		fillRect(img, 6, lightY, 4, 4, black)
		fillRect(img, 7, lightY + 1, 2, 2, lightColor)
	end
end

-- Tag Helper (same pattern as pressure_plate)
local tagDefs = {}
local function defineTag(name, fromFrame, toFrame)
	table.insert(tagDefs, { name = name, fromFrame = fromFrame, toFrame = toFrame })
end

local function newCelWithImage(frame)
	local img = Image(sprite.width, sprite.height)
	return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1

local function newFrame(duration)
	local frame
	if frameIndex == 1 then
		frame = sprite.frames[1]
	else
		frame = sprite:newEmptyFrame()
	end
	frame.duration = duration or 0.1
	frameIndex = frameIndex + 1
	return frame
end

-- ==================== CLOSED State ====================
local closedStart = frameIndex
local frameClosed = newFrame(1.0)
local celClosed = newCelWithImage(frameClosed)
drawFrame(celClosed.image)
drawPanel(celClosed.image, 44, lightRed)
defineTag("closed", closedStart, closedStart)

-- ==================== OPENING Animation ====================
local openingStart = frameIndex
local steps = 6
for i = 1, steps do
	local frame = newFrame(0.08)
	local cel = newCelWithImage(frame)
	drawFrame(cel.image)
	-- Linear interpolation: height 44 -> 0
	local progress = (i - 1) / (steps - 1)
	local h = math.floor(44 * (1 - progress))
	drawPanel(cel.image, h, lightRed)
end
defineTag("opening", openingStart, openingStart + steps - 1)

-- ==================== OPEN State ====================
local openStart = frameIndex
local frameOpen = newFrame(1.0)
local celOpen = newCelWithImage(frameOpen)
drawFrame(celOpen.image)
-- Door is gone, show green light on housing
fillRect(celOpen.image, 6, 1, 4, 2, black)
fillRect(celOpen.image, 7, 1, 2, 2, lightGreen)
defineTag("open", openStart, openStart)

-- ==================== CLOSING Animation ====================
local closingStart = frameIndex
for i = 1, steps do
	local frame = newFrame(0.08)
	local cel = newCelWithImage(frame)
	drawFrame(cel.image)
	-- Linear interpolation: height 0 -> 44
	local progress = (i - 1) / (steps - 1)
	local h = math.floor(44 * progress)
	drawPanel(cel.image, h, lightRed)
end
defineTag("closing", closingStart, closingStart + steps - 1)

-- Create all tags at the end
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
	app.alert("Switch Door Sprite Created!")
end
