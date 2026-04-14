from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
PUBLIC_DIR = ROOT / "public"

ICON_SIZES = [(16, 16), (32, 32), (48, 48), (64, 64)]
CANVAS_SIZE = 256
PADDING = 20
RADIUS = 58

GRADIENT_START = (77, 132, 120)
GRADIENT_END = (39, 81, 73)
TEXT_COLOR = (255, 247, 237, 255)
HIGHLIGHT_COLOR = (255, 255, 255, 55)


def build_gradient_tile() -> Image.Image:
    image = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    gradient = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    pixels = gradient.load()

    for y in range(CANVAS_SIZE):
        for x in range(CANVAS_SIZE):
            progress = (x + y) / ((CANVAS_SIZE - 1) * 2)
            color = tuple(
                round(start + (end - start) * progress)
                for start, end in zip(GRADIENT_START, GRADIENT_END)
            )
            pixels[x, y] = (*color, 255)

    mask = Image.new("L", (CANVAS_SIZE, CANVAS_SIZE), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(
        (PADDING, PADDING, CANVAS_SIZE - PADDING, CANVAS_SIZE - PADDING),
        radius=RADIUS,
        fill=255,
    )

    image.paste(gradient, mask=mask)

    overlay = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    overlay_draw = ImageDraw.Draw(overlay)
    overlay_draw.rounded_rectangle(
        (PADDING + 3, PADDING + 3, CANVAS_SIZE - PADDING - 3, CANVAS_SIZE - PADDING - 3),
        radius=RADIUS - 3,
        outline=HIGHLIGHT_COLOR,
        width=5,
    )
    image = Image.alpha_composite(image, overlay)

    return image


def draw_centered_text(image: Image.Image, text: str, font: ImageFont.FreeTypeFont) -> None:
    draw = ImageDraw.Draw(image)
    bbox = draw.textbbox((0, 0), text, font=font)
    width = bbox[2] - bbox[0]
    height = bbox[3] - bbox[1]
    x = (CANVAS_SIZE - width) / 2 - bbox[0]
    y = (CANVAS_SIZE - height) / 2 - bbox[1] - 6
    draw.text((x, y), text, font=font, fill=TEXT_COLOR)


def create_full_variant(font_path: str) -> Image.Image:
    image = build_gradient_tile()
    font = ImageFont.truetype(font_path, 96)
    draw_centered_text(image, "d20", font)
    return image


def create_simple_variant() -> Image.Image:
    image = build_gradient_tile()
    draw = ImageDraw.Draw(image)

    outer = [
        (128, 54),
        (58, 112),
        (77, 190),
        (128, 220),
        (179, 190),
        (198, 112),
    ]
    center_top = (128, 92)
    left_mid = (93, 129)
    right_mid = (163, 129)
    center_bottom = (128, 175)

    draw.line(outer + [outer[0]], fill=TEXT_COLOR, width=12, joint="curve")
    draw.line([outer[0], left_mid, outer[3], right_mid, outer[0]], fill=TEXT_COLOR, width=10, joint="curve")
    draw.line([outer[1], center_top, outer[5]], fill=TEXT_COLOR, width=10, joint="curve")
    draw.line([center_top, center_bottom], fill=TEXT_COLOR, width=10)
    draw.line([outer[2], left_mid], fill=TEXT_COLOR, width=10)
    draw.line([outer[4], right_mid], fill=TEXT_COLOR, width=10)

    return image


def save_outputs(name: str, image: Image.Image) -> None:
    png_path = PUBLIC_DIR / f"{name}.png"
    ico_path = PUBLIC_DIR / f"{name}.ico"
    image.save(png_path, format="PNG")
    image.save(ico_path, format="ICO", sizes=ICON_SIZES)


def main() -> None:
    PUBLIC_DIR.mkdir(exist_ok=True)
    font_path = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf"

    save_outputs("favicon-full", create_full_variant(font_path))
    save_outputs("favicon-simple", create_simple_variant())


if __name__ == "__main__":
    main()
