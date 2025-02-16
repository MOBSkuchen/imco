# imco
Image conversion tool, support lots of formats and also resizing.

Install using ``cargo install imco``
### Supported formats
- AVIF
- BMP
- DDS
- Farbfeld
- GIF
- HDR
- ICO
- JPEG
- EXR
- PNG
- PNM
- QOI
- TGA
- TIFF
- WebP
### Examples
Convert *lebron_james.png* to *lebron_james.ico*

``imco -i lebron_james.png -o lebron_james.ico``

Convert *lebron_james.jpg* to *lebron_james (tiff)*

``imco -i lebron_james.jpg -o lebron_james --output-format tiff``

Convert *lebron_james (ico)* to *lebron_james (tiff)*

``imco -i lebron_james -o lebron_james --output-format tiff --input-format ico``

Convert all pngs files under *images* to jpgs under *output*

``imco -i images/*.png -o output --output-format jpg --batch``