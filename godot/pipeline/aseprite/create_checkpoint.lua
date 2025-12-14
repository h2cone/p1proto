-- Aseprite Script: Checkpoint Generator
-- Creates a 16x24 checkpoint animation

-- Create new sprite
local sprite = Sprite(16, 24, ColorMode.RGB)
sprite.filename = "checkpoint.aseprite"

-- Define colors
local white = Color { r = 255, g = 255, b = 255, a = 255 }
local grey = Color { r = 100, g = 100, b = 100, a = 255 }
local darkGrey = Color { r = 60, g = 60, b = 60, a = 255 }
local red = Color { r = 230, g = 50, b = 50, a = 255 }
local darkRed = Color { r = 150, g = 30, b = 30, a = 255 }
local green = Color { r = 50, g = 230, b = 80, a = 255 }
local darkGreen = Color { r = 30, g = 150, b = 50, a = 255 }
local highlight = Color { r = 255, g = 255, b = 200, a = 100 } -- transparent highlight

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

-- Draw the pole and base (static elements)
local function drawPole(cel)
    -- Base
    fillRect(cel, 6, 22, 4, 2, darkGrey)
    fillRect(cel, 7, 21, 2, 1, grey)

    -- Pole
    fillRect(cel, 7, 2, 2, 20, white)
    -- Pole Topper
    fillRect(cel, 6, 1, 4, 1, white)
end

-- Draw the flag cloth
-- x, y: Top-left attachment point to the pole
-- color: Main color of the flag
-- waveOffset: Horizontal offset for waving effect (0, 1, or -1)
local function drawFlag(cel, y, color, shadeColor, waveOffset)
    local x = 9 -- Attached to right side of pole (pole is at x=7, width=2, so 7+2=9)
    local img = cel.image

    -- Flag shape (roughly rectangular with wave)
    local width = 7 -- Fits exactly from 9 to 15 (16px width)
    local height = 6

    for py = 0, height - 1 do
        for px = 0, width - 1 do
            -- Simple wave calculation
            local wave = 0
            if waveOffset ~= 0 then
                -- Shift pixels based on Y and offset
                if (py + px) % 3 == 0 then wave = waveOffset end
            end

            local drawX = x + px
            local drawY = y + py + wave

            -- Basic shading
            local c = color
            if py == height - 1 or px == width - 1 then
                c = shadeColor
            end

            -- Draw if within bounds
            if drawX < img.width and drawY < img.height then
                img:drawPixel(drawX, drawY, c)
            end
        end
    end
end

-- Create tag helper
local function createTag(name, fromFrame, toFrame)
    sprite:newTag(fromFrame, toFrame)
    sprite.tags[#sprite.tags].name = name
end

local function newCelWithImage(frame)
    local img = Image(sprite.width, sprite.height)
    return sprite:newCel(sprite.layers[1], frame, img)
end

local frameIndex = 1

-- ==================== UNCHECKED (Idle) ====================
-- 1 frame: Red flag at bottom
local uncheckedStart = frameIndex
local frame = sprite:newEmptyFrame(frameIndex)
frameIndex = frameIndex + 1
frame.duration = 0.5
local cel = newCelWithImage(frame)

drawPole(cel)
drawFlag(cel, 15, red, darkRed, 0) -- Low position

createTag("unchecked", uncheckedStart, frameIndex - 1)


-- ==================== RAISING (Transition) ====================
-- Flag moves up and turns green
local raisingStart = frameIndex
local steps = 6
local startY = 15
local endY = 3

for i = 0, steps - 1 do
    local frame = sprite:newEmptyFrame(frameIndex)
    frameIndex = frameIndex + 1
    frame.duration = 0.08
    local cel = newCelWithImage(frame)

    drawPole(cel)

    -- Linear interpolation for position
    local t = i / (steps - 1)
    local currentY = math.floor(startY + (endY - startY) * t)

    -- Color transition: Red until halfway, then Green
    local currentColor = (t < 0.5) and red or green
    local currentShade = (t < 0.5) and darkRed or darkGreen

    drawFlag(cel, currentY, currentColor, currentShade, (i % 2 == 0) and 1 or -1)
end

createTag("raising", raisingStart, frameIndex - 1)


-- ==================== CHECKED (Active) ====================
-- Loop: Green flag waving at top
local checkedStart = frameIndex
for i = 1, 4 do
    local frame = sprite:newEmptyFrame(frameIndex)
    frameIndex = frameIndex + 1
    frame.duration = 0.15
    local cel = newCelWithImage(frame)

    drawPole(cel)

    -- Waving animation
    local wave = 0
    if i == 1 then
        wave = 0
    elseif i == 2 then
        wave = 1
    elseif i == 3 then
        wave = 0
    elseif i == 4 then
        wave = -1
    end

    drawFlag(cel, 3, green, darkGreen, wave)
end

createTag("checked", checkedStart, frameIndex - 1)

app.alert("Checkpoint Sprite Created!")
