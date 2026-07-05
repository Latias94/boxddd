#include "box3d/box3d.h"

#include <emscripten/emscripten.h>
#include <stdbool.h>
#include <stdint.h>

EM_JS(int, boxddd_js_debug_take_error, (uint32_t token), {
  const errors = Module.boxdddDebugDrawErrors;
  if (!errors) return 0;
  const key = token >>> 0;
  const error = errors.get(key) | 0;
  errors.delete(key);
  return error;
});

EM_JS(uint32_t, boxddd_js_debug_shape_create, (uint32_t token, const b3DebugShape* shape), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_shape_create !== 'function') {
    setError(1);
    return 0;
  }
  try {
    return exports.boxddd_debug_shape_create(token >>> 0, shape >>> 0) >>> 0;
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug shape create failed: ${error}`);
    return 0;
  }
});

EM_JS(void, boxddd_js_debug_shape_destroy, (uint32_t token, uint32_t handle), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_shape_destroy !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_shape_destroy(token >>> 0, handle >>> 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug shape destroy failed: ${error}`);
  }
});

EM_JS(int, boxddd_js_debug_draw_shape,
      (uint32_t token, uint32_t handle, const b3WorldTransform* transform, b3HexColor color), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_shape !== 'function') {
    setError(1);
    return 0;
  }
  try {
    return exports.boxddd_debug_draw_shape(token >>> 0, handle >>> 0, transform >>> 0, color | 0) ? 1 : 0;
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw shape failed: ${error}`);
    return 0;
  }
});

EM_JS(void, boxddd_js_debug_draw_segment,
      (uint32_t token, const b3Pos* p1, const b3Pos* p2, b3HexColor color), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_segment !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_segment(token >>> 0, p1 >>> 0, p2 >>> 0, color | 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw segment failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_transform, (uint32_t token, const b3WorldTransform* transform), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_transform !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_transform(token >>> 0, transform >>> 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw transform failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_point,
      (uint32_t token, const b3Pos* position, float size, b3HexColor color), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_point !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_point(token >>> 0, position >>> 0, size, color | 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw point failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_sphere,
      (uint32_t token, const b3Pos* center, float radius, b3HexColor color, float alpha), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_sphere !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_sphere(token >>> 0, center >>> 0, radius, color | 0, alpha);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw sphere failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_capsule,
      (uint32_t token, const b3Pos* p1, const b3Pos* p2, float radius, b3HexColor color, float alpha), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_capsule !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_capsule(token >>> 0, p1 >>> 0, p2 >>> 0, radius, color | 0, alpha);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw capsule failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_bounds,
      (uint32_t token, const b3AABB* aabb, b3HexColor color), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_bounds !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_bounds(token >>> 0, aabb >>> 0, color | 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw bounds failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_box,
      (uint32_t token, const b3Vec3* extents, const b3WorldTransform* transform, b3HexColor color), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_box !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_box(token >>> 0, extents >>> 0, transform >>> 0, color | 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw box failed: ${error}`);
  }
});

EM_JS(void, boxddd_js_debug_draw_string,
      (uint32_t token, const b3Pos* position, const char* text, b3HexColor color), {
  const exports = Module.boxdddAppExports;
  const setError = (code) => {
    const key = token >>> 0;
    const errors = Module.boxdddDebugDrawErrors || (Module.boxdddDebugDrawErrors = new Map());
    if (!errors.has(key)) errors.set(key, code | 0);
  };
  if (!exports || typeof exports.boxddd_debug_draw_string !== 'function') {
    setError(1);
    return;
  }
  try {
    exports.boxddd_debug_draw_string(token >>> 0, position >>> 0, text >>> 0, color | 0);
  } catch (error) {
    setError(2);
    if (Module.printErr) Module.printErr(`boxddd debug draw string failed: ${error}`);
  }
});

static void* boxddd_create_debug_shape(const b3DebugShape* shape, void* context)
{
    uint32_t token = (uint32_t)(uintptr_t)context;
    uint32_t handle = boxddd_js_debug_shape_create(token, shape);
    return (void*)(uintptr_t)handle;
}

static void boxddd_destroy_debug_shape(void* userShape, void* context)
{
    uint32_t token = (uint32_t)(uintptr_t)context;
    uint32_t handle = (uint32_t)(uintptr_t)userShape;
    if (handle != 0) {
        boxddd_js_debug_shape_destroy(token, handle);
    }
}

static bool boxddd_draw_shape(void* userShape, b3WorldTransform transform, b3HexColor color, void* context)
{
    uint32_t token = (uint32_t)(uintptr_t)context;
    uint32_t handle = (uint32_t)(uintptr_t)userShape;
    return boxddd_js_debug_draw_shape(token, handle, &transform, color) != 0;
}

static void boxddd_draw_segment(b3Pos p1, b3Pos p2, b3HexColor color, void* context)
{
    boxddd_js_debug_draw_segment((uint32_t)(uintptr_t)context, &p1, &p2, color);
}

static void boxddd_draw_transform(b3WorldTransform transform, void* context)
{
    boxddd_js_debug_draw_transform((uint32_t)(uintptr_t)context, &transform);
}

static void boxddd_draw_point(b3Pos position, float size, b3HexColor color, void* context)
{
    boxddd_js_debug_draw_point((uint32_t)(uintptr_t)context, &position, size, color);
}

static void boxddd_draw_sphere(b3Pos center, float radius, b3HexColor color, float alpha, void* context)
{
    boxddd_js_debug_draw_sphere((uint32_t)(uintptr_t)context, &center, radius, color, alpha);
}

static void boxddd_draw_capsule(b3Pos p1, b3Pos p2, float radius, b3HexColor color, float alpha, void* context)
{
    boxddd_js_debug_draw_capsule((uint32_t)(uintptr_t)context, &p1, &p2, radius, color, alpha);
}

static void boxddd_draw_bounds(b3AABB aabb, b3HexColor color, void* context)
{
    boxddd_js_debug_draw_bounds((uint32_t)(uintptr_t)context, &aabb, color);
}

static void boxddd_draw_box(b3Vec3 extents, b3WorldTransform transform, b3HexColor color, void* context)
{
    boxddd_js_debug_draw_box((uint32_t)(uintptr_t)context, &extents, &transform, color);
}

static void boxddd_draw_string(b3Pos position, const char* text, b3HexColor color, void* context)
{
    boxddd_js_debug_draw_string((uint32_t)(uintptr_t)context, &position, text, color);
}

void boxddd_provider_debug_install_world_def(b3WorldDef* def, uint32_t registryToken)
{
    def->createDebugShape = boxddd_create_debug_shape;
    def->destroyDebugShape = boxddd_destroy_debug_shape;
    def->userDebugShapeContext = (void*)(uintptr_t)registryToken;
}

void boxddd_provider_debug_init_draw(b3DebugDraw* draw, uint32_t drawToken)
{
    draw->DrawShapeFcn = boxddd_draw_shape;
    draw->DrawSegmentFcn = boxddd_draw_segment;
    draw->DrawTransformFcn = boxddd_draw_transform;
    draw->DrawPointFcn = boxddd_draw_point;
    draw->DrawSphereFcn = boxddd_draw_sphere;
    draw->DrawCapsuleFcn = boxddd_draw_capsule;
    draw->DrawBoundsFcn = boxddd_draw_bounds;
    draw->DrawBoxFcn = boxddd_draw_box;
    draw->DrawStringFcn = boxddd_draw_string;
    draw->context = (void*)(uintptr_t)drawToken;
}

int boxddd_provider_debug_take_error(uint32_t token)
{
    return boxddd_js_debug_take_error(token);
}
