<h1 align="center">
    Black hole universal simulation interface engine
</h1>

<p align="center">
    A black hole ray tracer which can render in real time with meshes
</p>

![Imgur](https://i.imgur.com/ukPxaRV.png)

This project is a real-time black hole raytracer. It uses multiple techniques such as an adative grid to reduce calculations, bvh to render meshes, a relativity sphere algorithm to allow the mesh to be rendered in the scene with the black hole and many adjustable settings such as integration methods and parameters, object settings, black hole settings and more.

## Run
To run this project you can simply run with cargo. For performance reasons its preferable to run with the release option since it is level 3 optimized. Note the rust log enviroment variable is enabled by the program by default so you do not have to add it to get logs.
```sh
cargo run --release
```

To generate the disk texture you can run the sub project `perlin` although it has already been pre rendered for the project.
```sh
cargo run -p perlin
```

## build
```sh
cargo build --release
```

## Features
- [ ] black hole
    - [x] euler intergrator
    - [x] adaptive runge–kutta numerical intergrator
    - [x] accretion disk texture generation
    - [x] accretion red / blue shift
    - [x] relativity sphere
        - [x] feathering
    - [ ] multiple black holes
- [x] acceleration structures
    - [x] axis aligned bounding box
    - [x] bounding volume hierarchy
    - [x] adaptive grid for background stars
- [ ] looks
    - [x] bloom
    - [ ] reflections and scattering
    - [ ] pbr materials
- [x] Loading obj files
- [x] wgpu intergration
- [x] egui intergration


## UI (with egui)

The settings windows are all under menu > view.
Save a render under menu > save.
Full screen under menu > window > fullscreen or f11

![Imgur](https://i.imgur.com/TDLXCFW.png)


## Pipeline
The general flow of render passes is shown in the image below.
> Note: there are more passes now and this just demonstrates the ray and bloom passes.

![Imgur](https://i.imgur.com/JxbeT6H.png)
