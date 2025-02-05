A basic 3D renderer written in Rust. Capable of reading [OBJ files](https://en.wikipedia.org/wiki/Wavefront_.obj_file) containing either tris or quads.

Current features:
* Backface culling
* Frustum culling
* Textures via [MTL files](https://en.wikipedia.org/wiki/Wavefront_.obj_file#Material_template_library)
* Lighting
* Free-camera

Move the camera with WASD, left-shift, and space. Look around with the mouse.

To use with new files, change OBJECT_PATH in src/main to the path of the OBJ file. 
