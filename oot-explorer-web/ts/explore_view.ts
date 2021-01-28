import type * as Wasm from '../pkg';

import { $t } from './dollar_t';
import { ReflectView } from './reflect_view';
import { WasmModule } from './wasm';

interface DragState {
    offsetX: number;
    offsetY: number;
}

export class ExploreView {
    private readonly hexdump: Wasm.HexDumpView;
    private readonly reflect: ReflectView;
    public readonly element: HTMLElement;

    private x: number = 100;
    private y: number = 100;
    private dragState?: DragState = undefined;

    constructor(wasm: WasmModule, ctx: Wasm.Context) {
        let title;
        let closeButton;
        this.element = $t('div', {
            className: 'explore-view',
            children: [
                $t('div', {
                    className: 'explore-view-title-bar',
                    children: [
                        title = $t('div', {
                            className: 'explore-view-title',
                            textContent: 'Current Scene'
                        }),
                        closeButton = $t('div', {
                            className: 'explore-view-close',
                            // U+2573 BOX DRAWINGS LIGHT DIAGONAL CROSS
                            textContent: '\u{2573}'
                        }),
                    ]
                }),
            ]
        });
        title.addEventListener('mousedown', e => { this.handleTitleMouseDown(e); });
        // TODO: This needs to hook into a global event handler, or the drag will get stuck if the
        // cursor moves fast enough to escape the title bar in one update loop.
        title.addEventListener('mousemove', e => { this.handleTitleMouseMove(e); });
        title.addEventListener('mouseup', e => { this.handleTitleMouseUp(e); });
        closeButton.addEventListener('click', () => { this.handleCloseClick(); });

        this.element.appendChild((this.hexdump = new wasm.HexDumpView(document, ctx)).element);
        this.element.appendChild((this.reflect = new ReflectView(wasm, ctx)).element);
        this.reflect.onsethighlight = (start, end) => this.hexdump.setHighlight(start, end);
        this.reflect.onclearhighlight = () => this.hexdump.clearHighlight();

        this.updatePositionStyles();
    }

    updatePositionStyles() {
        this.element.style.left = this.x + 'px';
        this.element.style.top = this.y + 'px';
    }

    handleTitleMouseDown(e: MouseEvent) {
        this.dragState = {
            offsetX: this.x - e.clientX,
            offsetY: this.y - e.clientY,
        };

        // Move to the top of the window stack.
        let parent = this.element.parentElement!;
        parent.removeChild(this.element);
        parent.appendChild(this.element);
    }

    handleTitleMouseMove(e: MouseEvent) {
        if (this.dragState !== undefined) {
            this.x = e.clientX + this.dragState.offsetX;
            this.y = e.clientY + this.dragState.offsetY;
            this.updatePositionStyles();
        }
    }

    handleTitleMouseUp(e: MouseEvent) {
        this.dragState = undefined;
    }

    handleCloseClick() {
        this.element.parentElement!.removeChild(this.element);
    }
}
