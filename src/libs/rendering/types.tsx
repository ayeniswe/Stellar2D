import { TextureObject } from "../object/TextureObject"
type TexturesMapping = {
    [key: string]: TextureObject
}
type TextureSources = {
    [key: string]: HTMLImageElement
}
type TextureItem = {
    name: string;
    sx: number;
    sy: number;
    w: number;
    h: number;
}
type TextureItems = {
    [key: string]: TextureItem
}
type Textures = {
    name: string;
    src: string;
    groups: {
        [key: string]: TextureItems
    };
}
type Config = {
    name: string;
    version: string;
    textures: {
        [key: string]: {
            [key: string]: Textures; // String will follow a numbering format starting from 1..n
        };
    };
}
type RevisionAction = "added" | "removed"
type RevisionRecord = {
    texture: TextureObject,
    action: RevisionAction
}
type Texture = {
    /**
     * Adds a texture to the canvas.
     *
     * @param {boolean} clipping - Indicates whether clipping is applied.
     * @param {string} src - The location path of the texture or the `data/image` url.
     * @param {string} name - The name of the texture.
     * @param {number} x - The x-coordinate of the texture.
     * @param {number} y - The y-coordinate of the texture.
     * @param {number} w - The width of the texture.
     * @param {number} h - The height of the texture.
     * @param {number} sx - The source x-coordinate of the texture.
     * @param {number} sy - The source y-coordinate of the texture.
     * @param {number} l - The layer of the texture.
     * @returns {number[]} Returns an array containing the x and y coordinates of the removed texture, or an empty array if the texture was not removed.
     *
     * @description
     * This method retrieves the brush information.
     * It then scales the x and y coordinates based on the dimensions of the texture.
     * 
     * NOTE: scaling only applies if clipping mode is on
     * 
     * If the texture does not already exist on the canvas, it retrieves the respective values from the config file.
     * It then stores the texture in the textures mapping using the destination coordinates and dimensions as the key.
     * Additionally, it stores an action object in the revisions array to keep track of the added texture.
     */
    addTexture: (src: string, name: string, clipping: boolean, x: number, y: number, w: number, h: number, sx?: number, sy?: number, l?: number) => number[]
    /**
     * Removes a texture from the canvas.
     *
     * @param {number} x - The x-coordinate of the texture.
     * @param {number} y - The y-coordinate of the texture.
     * @param {number} l - The layer of the texture.
     *
     * @description
     * This method retrieves the texture information.
     * from the pixel boundings if the texture exists.
     */
    removeTexture: (x: number, y: number, l?: number) => void,
    /**
     * Undoes the last revision made to the canvas.
     *
     * @returns {boolean} Returns `true` if a revision was successfully undone, or `false` if there were no revisions to undo.
     *
     * @description
     * This method checks if there are any revisions in the `this.__revisions` array by checking its length.
     * If there are revisions, it pops the last revision from the array using the stack method.
     * The code then checks if there is a valid `action` object obtained from the popped revision.
     * Based on the action, it performs different actions:
     * - `added`: it deletes the corresponding texture from the textures mapping and clears the canvas in that area.
     * - `removed`: it adds the corresponding texture mapping back to the textures mapping.
     * 
     * NOTE: This method is only available to undo "added" actions
     */
    undoRevision: () => void,
    /**
    * Removes all textures from the canvas.
    */
    removeAllTexture: () => void
    /**
    * This mehtod is responsible for rendering textures onto a canvas.
    *
    * Description:
    * This method is used to render textures onto a canvas.
    * It retrieves the textures from the textures mapping and decides if
    * the texture source image is already in cache to render from otherwise it will be loaded as a new image.
    * 
    * NOTE: The load a new image is typically done from drag and drop events
    */
    render: () => void
    /**
    * Initializes the renderer.
    */
    initialize: () => Promise<void>
    /**
     * Selects a texture from the canvas to drag and drop.
     * @returns {string} Returns the canvas data url for background image when dragging
     */
    selectTexture: (x: number, y: number, l?: number) => void
    textureRenderer: {
        /**
        * All textures sources available to be used in the renderer
        */
        textureSources: TextureSources
        /**
        * The canvas for the scene to be rendered.
        */
        readonly ctx: CanvasRenderingContext2D
    }
}
export type {
    Texture,
    TextureSources,
    TextureItem,
    TextureItems,
    Config,
    TexturesMapping,
    RevisionAction,
    RevisionRecord,
    Textures,
}