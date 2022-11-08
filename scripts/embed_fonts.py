"""Takes an SVG file and embeds all of its fonts"""

import argparse
import base64
import re

import requests
from lxml import etree

URL_REGEX = re.compile(
    r"https:\/\/(www\.)?[a-zA-Z]+\.[a-z]+/([a-zA-Z]+)\.[a-z\d]+"
)  # forgive me for the hacky regex


FONT_TEMPLATE = """
@font-face {{
    font-family: {name};
    src: url(data:font/woff2;base64,{data});
}}
"""


def main() -> None:
    parser = argparse.ArgumentParser(description="Embed all fonts in a given SVG file")
    parser.add_argument(
        "filename", type=str, help="The filename for the SVG to process"
    )

    args = parser.parse_args()
    filename = args.filename

    tree = etree.parse(filename)
    root = tree.getroot()

    for element in root.iter("{http://www.w3.org/2000/svg}style"):
        embedded_fonts: list[str] = []
        for url_match in URL_REGEX.finditer(element.text):
            url = url_match.group(0)
            font_name = url_match.group(2)
            
            data = requests.get(url).content
            encoded = base64.b64encode(data)
            
            font_str = FONT_TEMPLATE.format(name=font_name, data=encoded.decode())
            embedded_fonts.append(font_str)

        element.text = "\n".join(embedded_fonts)

    tree.write(filename + ".foo.svg")


if __name__ == "__main__":
    main()
