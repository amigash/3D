A basic 3D renderer written in Rust. Capable of reading [OBJ files](https://en.wikipedia.org/wiki/Wavefront_.obj_file) containing either tris or quads.

Current features:
* Backface culling
* Frustum culling
* Textures via [MTL files](https://en.wikipedia.org/wiki/Wavefront_.obj_file#Material_template_library)
* Lighting
* Free-camera

# Installation and running

```git clone https://github.com/amigash/3D/```

```cd 3D```

```cargo run --release``` 

(release mode is important for performance)

Move the camera with WASD, left-shift, and space. Look around with the mouse.

To use with new files, change OBJECT_PATH in src/main to the path of the OBJ file. 

If the program runs slowly, try increasing SCALE in src/main (higher means more downscaling). Similarly, decrease SCALE for better visual quality (maximum at SCALE = 1).
