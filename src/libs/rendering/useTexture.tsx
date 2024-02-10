import { Config, RevisionRecord, TexturesMapping, TextureSources } from './types';
import { useSignal } from '@preact/signals-react';
import { getHeight, getWidth } from '../../utils/styleProps';
import { Sprite, Tile } from '../object';
import { TextureObject } from '../object/TextureObject';
import { SCENE } from '../../features/Scene';
const useTexture = (ctx: CanvasRenderingContext2D) => {
    const revisions = useSignal<RevisionRecord[]>([]);
    const sources = useSignal<TextureSources>({});
    const mapping = useSignal<TexturesMapping>({});
    const selector = useSignal<TextureObject | undefined>(undefined);
    const pixels = new Map<string, TextureObject>();
    const CONFIG: Config = process.env.NODE_ENV === 'production' ? require('../../data/config.json') : require('../../data/test-config.json');
    const textureRenderer = {
        get textureSources() {
            return sources.value;
        },
        get ctx() {
            return ctx;
        }
    }
    /**
    * Initializes the renderer.
    */
    const initialize = async () => {
        await addAllSources();
        startListeners();
    }
    /**
     * Adds all texture sources to the canvas.
     *
     * @returns {Promise<void>} Returns a promise that resolves when all texture sources have been added.
     *
     * @description
     * This method iterates over each textures type and texture within the textures type
     * For each texture, it retrieves the name and src (where file is found relative to the path).
     * It adds an entry to the textures sources
     * 
     * NOTE: All textures should be loaded before dealing with application
     * 
     * @todo Allow application use while loading in background using web workers
     */
    const addAllSources = async (): Promise<void> => {
        for (const key in CONFIG.textures) {
            const total = Object.keys(CONFIG.textures[key]).length;
            for (let index = 1; index <= total; index++) {
                const texture = CONFIG.textures[key][index];
                const name = texture.name;
                const path = `assets/textures/${texture.src}`;
                sources.value = {
                    ...sources.value,
                    [name]: new Image(),
                };
                try {
                    const url: string = (await import(`../../${path}`)).default;
                    sources.value[name].src = url;
                    sources.value[name].onload = () => {
                        console.log(`Loaded "${name}" texture - ${index} of ${total}`);
                    };
                    sources.value[name].onerror = () => console.error(`Failed to load "${name}" texture from ${path}`);
                } catch {
                }
            }
        }
    }
    /**
     * Checks if an position exists in texture mapping.
     *
     * @param {number} x The x-coordinate of the position.
     * @param {number} y The y-coordinate of the position.
     * @param {number} l The actual layer of the position.
     * @return {boolean} Returns true if the position exists, false otherwise.
     * 
     * NOTE: The key is generated from the destination coordinates (x and y) and dimensions (height and width).
     */
    const posExists = (x: number, y: number, l: number): boolean => {
        return mapping.value[ckey(x, y, l)] ? true : false;
    }
    /**
     * Scales the coordinates based on the canvas size
     *
     * @param {number} x The x coordinate.
     * @param {number} y The y coordinate.
     * @param {number} w The width.
     * @param {number} h The height.
     * @param {boolean} clipping - Whether clipping is enabled or not.
     * @param {boolean} l - The actual layer of the position.
     * @return {number[]} The scaled coordinates.
     */
    const scaling = (x: number, y: number, w: number, h: number, clipping: boolean, l: number): number[] => {
        const inBoundsY = (h: number, y: number) => {
            return y >= ctx.canvas.height - h ? ctx.canvas.height - h : y < 0 ? 0 : y;
        }
        const inBoundsX = (w: number, x: number) => {
            return x >= ctx.canvas.width - w ? ctx.canvas.width - w : x < 0 ? 0 : x;
        }
        const closestUnit = (val: number, scale: number) => {
            return Math.min(Math.abs(scale - val), val);
        }
        const clipX = (w: number, x: number) => {
            return (x - closestUnit(x % w, w)) % w === 0 ? x - closestUnit(x % w, w) : closestUnit(x % w, w) + x;
        }
        const clipY = (h: number, y: number) => {
            return (y - closestUnit(y % h, h)) % h === 0 ? y - closestUnit(y % h, h) : closestUnit(y % h, h) + y;
        }

        let dx, dy;
        if (clipping) {
            dx = clipX(w, inBoundsX(w, x));
            dy = clipY(h, inBoundsY(h, y));
            if (posExists(dx, dy, l) && x < dx) dx = inBoundsX(w, dx - w);
            if (posExists(dx, dy, l) && y < dy) dy = inBoundsY(w, dy - h);
        } else {
            dx = inBoundsX(w, x);
            dy = inBoundsY(h, y);
        }
        return [dx, dy];
    }
    /**
     * Clears any part of the canvas.
     * 
     * @param {number} dx - The x-coordinate of the top-left corner of the area.
     * @param {number} dy - The y-coordinate of the top-left corner of the area.
     * @param {number} w - The width of the area.
     * @param {number} h - The height of the area.
     */
    const clearCanvas = (dx: number, dy: number, w: number, h: number) => {
        ctx.clearRect(dx, dy, w, h);
    }
    /**
     * Generates a unique key for a texture based on the position and layer.
     * @param {number} dx - The x-coordinate of the top-left corner of the area.
     * @param {number} dy - The y-coordinate of the top-left corner of the area.
     * @param {number} layer - The layer the texture is on.
     */
    const ckey = (dx: number, dy: number, layer: number = 1) => {
        return `${dx},${dy}-${layer}`;
    }
    /**
     * Get the topmost pixel layer
     */
    const getTopLayer = (x: number, y: number, l: number): TextureObject | undefined => {
        if (!pixels.get(ckey(x, y, l))) {
            return pixels.get(ckey(x, y, l-1));
        }
       return getTopLayer(x, y, l+1);
    }
    /**
     * Check rect collison of known pixel region
     */
    const inPixelRegion = (x: number, y: number, w: number, h: number, l: number): boolean => {
        if (pixels.get(ckey(x, y, l))
        || pixels.get(ckey(x+w, y, l))
        || pixels.get(ckey(x+w, y+h, l))
        || pixels.get(ckey(x, y+h, l))) {
            return true;
        }
        return false;
    }
    /**
     * Add to pixels mapping for texture selection bounding box
     */
    const addPixels = (texture: TextureObject) => {
        const { l, w, h, dx, dy } = texture;
        for (let i = dx; i <= dx+w; i++) {
            for (let j = dy; j <= dy+h; j++) {
                pixels.set(ckey(i, j, l), texture);
            }
        }
    }
    const removePixels = (texture: TextureObject) => {
        const { l, w, h, dx, dy } = texture;
        // if (!pixels.get(ckey(dx, dy, l))) return;
        for (let i = dx; i <= dx+w; i++) {
            for (let j = dy; j <= dy+h; j++) {
                pixels.delete(ckey(i, j, l));
            }
        }
    }
    const addTexture = (src: string, name: string, clipping: boolean, x: number, y: number, w: number, h: number, sx = 0, sy = 0, l = 1): number[] => {
        console.log(l)
        // Scaling and account for clipping if true
        const [dx, dy] = scaling(x, y, w, h, clipping, l);
        if (inPixelRegion(x, y, w, h, l)) {
            // Recuriselvy move up layer if layer is already existing
            addTexture(src, name, clipping, dx, dy, w, h, sx, sy, l+1);
            return [];
        } else {
            // Store the texture by the name
            let texture;
            if (sx !== 0 || sy !== 0) {
                const preloadTexture = sources.value[src];
                texture = new Tile(ctx, preloadTexture, name, dx, dy, w, h, sx, sy, l);
            } else {
                texture = new Sprite(ctx, src, name, dx, dy, w, h, l);
            }
            // Store pixel mapping for selection
            addPixels(texture);
            mapping.value[ckey( dx, dy, l )] = texture;
            // Store action in history
            revisions.value.push({
                texture: texture,
                action: "added"
            });
            return [dx, dy];
        }
    }
    const removeTexture = (x: number, y: number, l = 1) => {
        if (pixels.get(ckey(x, y, l))) {
            // Recuriselvy move up layer if layer is already existing
            removeTexture(x, y, l+1);
            return [];
        } else {
            const texture = pixels.get(ckey(x, y, l-1))
            if (!texture) return [];
            const {dx, dy, w, h} = texture;
            delete mapping.value[ckey(dx, dy, l-1)];
            removePixels(texture);
            clearCanvas(dx, dy, w, h);
        }
    }
    const undoRevision = () => {
        const action = revisions.value.pop();
        if (action) {
            const { l, dx, dy, w, h } = action.texture;
            switch (action.action) {
                case "added":
                    removePixels(action.texture);
                    delete mapping.value[ckey( dx, dy, l)];
                    clearCanvas(dx, dy, w, h);
                    break;
                default:
                    break;
            }
        }
    }
    const render = () => {
        for (const key in mapping.value) {
            const texture = mapping.value[key];
            texture.render();
        }
    }
    const removeAllTexture = () => {
        mapping.value = {};
        clearCanvas(0, 0, ctx.canvas.width, ctx.canvas.height);
    }
    const selectTexture = async (x: number, y: number, l: number = 1) => {
        if (!selector.value) {
            // Check if texture exists in pixel boundings - at topmost layer
            selector.value = getTopLayer(x, y, l);
            if (selector.value) {
                document.getElementById(SCENE.CANVAS)!.style.cursor = 'move';
            }
        } else {
            const texture = selector.value;
            if (!texture) return;
            // Remove old pixel boundings and position
            const {l, w, h, dx, dy} = texture;
            removePixels(texture);
            // console.log(mapping.value[ckey(dx, dy, l)])
            delete mapping.value[ckey(dx, dy, l)];
            // Update object new location
            texture.dx = x;
            texture.dy = y;
            // Redraw canvas
            clearCanvas(dx, dy, w, h);
            ctx.drawImage(texture.texture.canvas, x, y);
            document.onmouseup = (e) => {
                // Add new pixel boundings and position
                addPixels(texture);
                mapping.value[ckey(texture.dx, texture.dy, l)] = texture;
                // Reset canvas cursor
                document.onmouseup = null;
                document.getElementById(SCENE.CANVAS)!.style.cursor = 'pointer';
                selector.value = undefined;
            }
        }
    }
    /**
     * Add a listener to listen for various rendering events
     */
    const startListeners = () => {
        // Resizing application window
        window.addEventListener("resize", () => {
            ctx.canvas.width = getWidth(ctx.canvas).toNumber();
            ctx.canvas.height = getHeight(ctx.canvas).toNumber();
            render();
        })
    }
    return {
        initialize,
        addTexture,
        selectTexture,
        removeTexture,
        undoRevision,
        render,
        removeAllTexture,
        textureRenderer,
    };
}
export {
    useTexture
}