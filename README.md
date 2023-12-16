3D OBB with pointer-based manipulation features.

![demo](https://github.com/bevy_fsl_box_frame/images/demo.gif)

We say "frame" because only the 12 edges of the box are rendered via
`bevy_polyline`.

Faces of the box can be dragged by the pointer to manipulate the box extents.
As the pointer hovers over each face, visual feedback is provided (highlight
material).

Depends on [`bevy_mod_picking::DefaultPickingPlugins`] and
[`bevy_polyline::PolylinePlugin`].
