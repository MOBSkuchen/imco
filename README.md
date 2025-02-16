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

``imco lebron_james.png lebron_james.ico`` or ``imco lebron_james.png --output-format ico``

Convert *lebron_james.jpg* to *lebron_james (tiff)*

``imco lebron_james.jpg lebron_james --output-format tiff``

Convert *lebron_james (ico)* to *lebron_james (tiff)*

``imco lebron_james lebron_james --output-format tiff --input-format ico``

Convert all pngs files under *images* to jpgs under *output*

``imco images/*.png output --output-format jpg --batch``