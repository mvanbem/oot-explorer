import { clamp } from './math';
import { $t } from './dollar_t';

export const WindowManager = (() => {
    class WindowManager {
        private readonly windows: WMWindow[] = [];

        constructor() {
            window.addEventListener('resize', () => this.handleResize());
        }

        add(window: WMWindow) {
            this.windows.push(window);
        }

        handleResize() {
            console.log('repositioning all windows');
            for (let window of this.windows) {
                window.updatePosition();
            }
        }
    }

    return new WindowManager();
})();

interface WindowOptions {
    title?: string;
    cannotClose?: boolean;
    width?: number;
    height?: number;
}

interface DragState {
    offsetX: number;
    offsetY: number;
}

export class WMWindow {
    public readonly element: HTMLElement;
    private readonly resizeHandleGlobalMouseMoveHandler: (e: MouseEvent) => void =
        e => this.handleResizeHandleMouseMove(e);
    private readonly resizeHandleGlobalMouseUpHandler: () => void =
        () => this.handleResizeHandleMouseUp();
    private readonly titleGlobalMouseMoveHandler: (e: MouseEvent) => void =
        e => this.handleTitleMouseMove(e);
    private readonly titleGlobalMouseUpHandler: () => void =
        () => this.handleTitleMouseUp();
    private readonly hasCloseButton: boolean;

    // TODO: Have the WindowManager assign cascading new-window coordinates.
    private x: number = 100;
    private y: number = 100;
    private width: number = 400;
    private height: number = 300;
    private resizeHandleDragState?: DragState = undefined;
    private titleDragState?: DragState = undefined;

    constructor({ title, cannotClose, width, height }: WindowOptions) {
        this.hasCloseButton = !cannotClose;
        let resizeHandle;
        this.element = $t('div', {
            className: 'window',
            children: [
                resizeHandle = $t('div', { className: 'window-resize-handle' }),
            ]
        });
        resizeHandle.addEventListener('mousedown', e => this.handleResizeHandleMouseDown(e));

        if (title !== undefined) {
            let titleElement;
            let titleBar = $t('div', {
                className: 'window-title-bar',
                children: [
                    titleElement = $t('div', { className: 'window-title', textContent: title }),
                ],
            });
            this.element.appendChild(titleBar);
            titleElement.addEventListener('mousedown', e => this.handleTitleMouseDown(e));

            if (!cannotClose) {
                let close = $t('div', {
                    className: 'window-close',
                    // U+2573 BOX DRAWINGS LIGHT DIAGONAL CROSS
                    textContent: '\u{2573}',
                });
                titleBar.appendChild(close);
                close.addEventListener('click', e => { this.close() });
            }
        }

        if (width !== undefined) {
            this.width = width;
        }
        if (height !== undefined) {
            this.height = height;
        }

        this.updatePosition();
    }

    close() {
        this.element.parentElement!.removeChild(this.element);
        // TODO: tell the WindowManager?
    }

    /**
     * Enforces position/sizing limits and sets element CSS properties.
     */
    updatePosition() {
        // The parent element is not set the first time this function is called.
        if (this.element.parentElement) {
            let parentRect = this.element.parentElement!.getBoundingClientRect();
            let thisRect = this.element.getBoundingClientRect();

            // Prevent the title bar from leaving the screen.
            const MARGIN = 32;
            const CLOSE_BUTTON_WIDTH = 32;
            this.x = clamp(
                this.x,
                MARGIN - thisRect.width + (this.hasCloseButton ? CLOSE_BUTTON_WIDTH : 0),
                parentRect.width - MARGIN);
            this.y = clamp(
                this.y,
                0,
                parentRect.height - MARGIN);

            // Prevent the window from being bigger than the screen.
            this.width = clamp(
                this.width,
                MARGIN + (this.hasCloseButton ? CLOSE_BUTTON_WIDTH : 0),
                parentRect.width);
            this.height = clamp(
                this.height,
                MARGIN,
                parentRect.height);
        }

        this.element.style.left = this.x + 'px';
        this.element.style.top = this.y + 'px';
        this.element.style.width = this.width + 'px';
        this.element.style.height = this.height + 'px';

        // Only fire events for updates after the constructor has returned.
        if (this.element.parentElement) {
            this.onResize();
        }
    }

    handleTitleMouseDown(e: MouseEvent) {
        e.preventDefault();

        this.titleDragState = {
            offsetX: this.x - e.clientX,
            offsetY: this.y - e.clientY,
        };

        // Move to the top of the window stack.
        let parent = this.element.parentElement!;
        parent.removeChild(this.element);
        parent.appendChild(this.element);

        // Attach the global mouse event handlers.
        document.addEventListener('mousemove', this.titleGlobalMouseMoveHandler);
        document.addEventListener('mouseup', this.titleGlobalMouseUpHandler);
    }

    handleTitleMouseMove(e: MouseEvent) {
        if (this.titleDragState !== undefined) {
            this.x = e.clientX + this.titleDragState.offsetX;
            this.y = e.clientY + this.titleDragState.offsetY;
            this.updatePosition();
        }
    }

    handleTitleMouseUp() {
        this.titleDragState = undefined;

        // Remove the global mouse event handlers.
        document.removeEventListener('mousemove', this.titleGlobalMouseMoveHandler);
        document.removeEventListener('mouseup', this.titleGlobalMouseUpHandler);
    }

    handleResizeHandleMouseDown(e: MouseEvent) {
        e.preventDefault();

        this.resizeHandleDragState = {
            offsetX: this.width - e.clientX,
            offsetY: this.height - e.clientY,
        };

        // Move to the top of the window stack.
        let parent = this.element.parentElement!;
        this.onBeforeReattach();
        parent.removeChild(this.element);
        this.onAfterReattach();
        parent.appendChild(this.element);

        // Attach the global mouse event handlers.
        document.addEventListener('mousemove', this.resizeHandleGlobalMouseMoveHandler);
        document.addEventListener('mouseup', this.resizeHandleGlobalMouseUpHandler);
    }

    handleResizeHandleMouseMove(e: MouseEvent) {
        if (this.resizeHandleDragState !== undefined) {
            this.width = e.clientX + this.resizeHandleDragState.offsetX;
            this.height = e.clientY + this.resizeHandleDragState.offsetY;
            this.updatePosition();
        }
    }

    handleResizeHandleMouseUp() {
        this.resizeHandleDragState = undefined;

        // Remove the global mouse event handlers.
        document.removeEventListener('mousemove', this.resizeHandleGlobalMouseMoveHandler);
        document.removeEventListener('mouseup', this.resizeHandleGlobalMouseUpHandler);
    }

    /// This is intended to be overridden by extending classes.
    protected onResize() { }

    /// This is intended to be overridden by extending classes.
    protected onBeforeReattach() { }

    /// This is intended to be overridden by extending classes.
    protected onAfterReattach() { }
}
