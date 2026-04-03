import zipfile, os, shutil
from PIL import Image

jar = os.path.join(os.environ["APPDATA"], ".minecraft", "versions", "1.16.5", "1.16.5.jar")
out_vanilla = os.path.join(os.path.dirname(__file__), "_vanilla_mc165_minecraft.png")

with zipfile.ZipFile(jar, "r") as z:
    with z.open("assets/minecraft/textures/gui/title/minecraft.png") as src, open(out_vanilla, "wb") as dst:
        dst.write(src.read())

im = Image.open(out_vanilla)
print(f"vanilla 1.16.5 minecraft.png  size={im.size}  mode={im.mode}")

rp_png = os.path.join(
    os.environ["APPDATA"],
    r".minecraft\instancias\Paraguacraft_1.16.5_Vanilla\resourcepacks"
    r"\ParaguacraftBrandPack\assets\minecraft\textures\gui\title\minecraft.png"
)
if os.path.exists(rp_png):
    im2 = Image.open(rp_png)
    print(f"our resourcepack minecraft.png  size={im2.size}  mode={im2.mode}")
else:
    print("resourcepack minecraft.png NOT FOUND at", rp_png)
