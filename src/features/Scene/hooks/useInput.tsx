import configuration from '../../../data/test-config.json';
import { Bindings } from "../../../libs/input";
import { useSignal } from "@preact/signals-react";
import { Config, Texture } from "../../../libs/rendering";
import { Brush } from "./type";
import { iconEffects } from "../../../libs/effects";
import { SCENE } from "..";
/**
 * The hook is responsible for managing 
 * the scene user interaction and keybindings and 
 * provides a way to set and interact with the selected brush.
 * 
 * @param renderer - The texture renderer
 * @param brush - The selected brush
 * 
 */
const useInput = (renderer: Texture) => {
    const config: Config = configuration;
    const __editable = useSignal(false);
    const __drag = useSignal(false);
    const __safety = useSignal(true);
    const __trash = useSignal(false);
    const __clip = useSignal(false);
    const __ready = useSignal(false);
    const _brush = useSignal<Brush | null>(null);
    const { applyDragEffect, applyTrashEffect, applyClippingEffect, applyEditingEffect } = iconEffects();
    const input = {
        get editable() {
            return __editable.value
        },
        set editable(value: boolean) {
            __editable.value = value
        },
        get clip() {
            return __clip.value
        },
        set clip(value: boolean) {
            __clip.value = value
        },
        get trash() {
            return __trash.value
        },
        set trash(value: boolean) {
            __trash.value = value
        },
        get ready() {
            return __ready.value
        },
        set ready(value: boolean) {
            __ready.value = value
        },
        get safety() {
            return __safety.value
        },
        set safety(value: boolean) {
            __safety.value = value
        },
        get drag() {
            return __drag.value
        },
        set drag(value: boolean) {
            __drag.value = value
        },
        get brush() {
            return _brush.value
        },
        set brush(value: Brush | null) {
            _brush.value = value
        }
    }
    const initialize = (): void => {
        const bindings = Bindings.getInstance();
        bindings.addBinding(handleDropSprite.bind(this), [], 'drop', false, SCENE.CANVAS);
        bindings.addBinding(handleBrush.bind(this), [], 'mousemove', false, SCENE.CANVAS);
        bindings.addBinding(handleDrawing.bind(this), ['LeftButton'], ['mousedown', 'mousemove'], false, SCENE.CANVAS);
        bindings.addBinding(toggleClipMode.bind(this), ['c'], 'keydown', true);
        bindings.addBinding(toggleDragMode.bind(this), ['d'], 'keydown', true);
        bindings.addBinding(toggleEditMode.bind(this), ['e'], 'keydown', true);
        bindings.addBinding(toggleTrashMode.bind(this), ['Delete'], 'keydown', true);
        bindings.addBinding(clearCanvas.bind(this), ['Control','a'], 'keydown', true);
        bindings.addBinding(handleUndo.bind(this), ['Control', 'z'], 'keydown', false);
    }
    const removeAll = () => {
        renderer.removeAllTexture();
    }
    /**
     * Handles the ability to undo actions, such as adding a tile onto the scene.
     *
     * NOTE: This method is only available if the scene is ready.
     * 
     * NOTE: The method does not allow redo, such as undoing a previous undo action.
     */
    const handleUndo = (): void => {
        renderer.undoRevision()
        renderer.render();
    }
    /**
     * Handles the constant creation of a div to
     * create a brush effect.
     */
    const handleBrush = (event: MouseEvent): void => {
        if (!_brush.value) return;
        const brushElement = document.getElementById(SCENE.BRUSH)!;
        if (__editable.value) {
            const { w, h } = _brush.value.object;
            brushElement.style.left = `${event.clientX}px`;
            brushElement.style.top = `${event.clientY}px`;
            brushElement.style.display = 'flex';
            brushElement.style.width = `${w}px`;
            brushElement.style.height = `${h}px`;
        } else {
            brushElement.style.display = 'none';
        }
    }
    /**
     * Handles the drawing action in the level scene based on mouse events.
     * If the scene is in `trash` mode, it removes an element at the specified position.
     * If the scene is in `selection` mode "when edit it turned off", it allows the selection of an element and drop it at the specified position.
     * If the scene is in `drag` mode, it rapidly adds an element at the specified position.
     * @param {MouseEvent} event - The mouse event object.
     * 
     * NOTE: This method is only available if the scene is ready.
     */
    const handleDrawing = (event: MouseEvent): void => {
        if (!__editable.value && !__trash.value) {
            // Select texture
            renderer.selectTexture(event.offsetX, event.offsetY);
        } else {
            if (!__ready.value || !_brush.value || !__drag.value && event.type === "mousemove") return;
            // TODO: make more dynamic way to do this
            // Get brush src image tilesheet data
            const { id, object } = _brush.value!;
            const { name, h, w, sx, sy } = object;
            const src = config.textures["tilesets"][id.split("-")[0]].name;
            // Remove or add a texture
            if (__trash.value) {
                renderer.removeTexture(event.offsetX, event.offsetY);
            } else {
                renderer.addTexture(src, name, __clip.value, event.offsetX, event.offsetY, w, h, sx, sy);
            }
        }
        renderer.render();
    }
    /**
     * Handles the dropping of a sprite onto the scene.
     * 
     */
    const handleDropSprite = (e: DragEvent): void => {
        e.preventDefault();
        const frames = JSON.parse(e.dataTransfer!.getData("application/sprite")).frames;
        renderer.addTexture(frames[0].src, "", __clip.value, e.offsetX, e.offsetY, frames[0].w, frames[0].h);
        renderer.render();
    }
    // *****************************************
    //        TOGGLE HANDLERS SECTION
    // *****************************************
    // The handlers below are used
    // to toggle the state of the scene
    // modes and apply/remove effects respectively
    //
    // NOTE: These methods are only available if
    // the scene is ready
    // *****************************************
    const toggleTrashMode = () => {
        const button = document.getElementById(SCENE.TRASH);
        if (!button) return;
        if (__trash.value) {
            __trash.value = false;
            applyTrashEffect(button, false);
        } else {
            __trash.value = true;
            applyTrashEffect(button);
        }
    }
    const toggleEditMode = () => {
        const button = document.getElementById(SCENE.EDIT);
        if (!__ready.value || !button) return;
        if (__editable.value) {
            __editable.value = false;
            applyEditingEffect(button, false);
        } else {
            __editable.value = true;
            applyEditingEffect(button);
        }
    }
    const toggleClipMode = () => {
        const button = document.getElementById(SCENE.CLIP);
        if (!button) return;
        if (__clip.value) {
            __clip.value = false;
            applyClippingEffect(button, false);
        } else {
            __clip.value = true;
            applyClippingEffect(button);
        }
    }
    const toggleDragMode = () => {
        const button = document.getElementById(SCENE.DRAG);
        if (!__ready.value || !button) return;
        if (__drag.value) {
            __drag.value = false;
            applyDragEffect(button, false);
        } else {
            __drag.value = true;
            applyDragEffect(button);
        }
    }
    /**
     * Handles the clearing of the entire canvas in the scene.
     * The safety pin is turned off for 3 seconds.
     * After 3 seconds, the safety pin is turned back on to prevent serious actions.
     *
     * NOTE: This method is only available if the editor is ready.
     */
    const clearCanvas = () => {
        toggleTrashMode();
        __safety.value = false;
        setTimeout(() => {
            toggleTrashMode();
            __safety.value = true;
        }, 3000);
    }
    return {
        input,
        removeAll,
        initialize
    }
}
export {
    useInput
}