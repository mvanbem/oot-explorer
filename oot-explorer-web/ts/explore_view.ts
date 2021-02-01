import type * as Wasm from '../pkg';

import { $t } from './dollar_t';
import { ItemView } from './item_view';
import { ReflectView } from './reflect_view';
import { WasmModule } from './wasm';
import { WMWindow } from './window_manager';

interface Selection {
    item: ItemView;
    start: number;
    end: number;
}

interface Highlight {
    start: number;
    end: number;
}

export class ExploreView extends WMWindow {
    private readonly hexdumpContainer: HTMLElement;
    private readonly hexdump: Wasm.HexDumpView;
    private readonly reflect: ReflectView;

    private lastScroll: number = 0;
    private selection?: Selection;
    private highlight?: Highlight;
    private refreshMarkingsScheduled: boolean = false;

    constructor(wasm: WasmModule, ctx: Wasm.Context, root: Wasm.ReflectRoot) {
        // TODO: Don't hard-code the title.
        super({ title: 'Explore ' + root.description, width: 464 });

        this.hexdumpContainer = $t('div', { className: 'explore-view-hexdump' });
        this.element.appendChild(this.hexdumpContainer);
        this.hexdumpContainer.addEventListener('scroll', () => this.hexdump.regenerateChildren());

        this.hexdump = new wasm.HexDumpView(document, ctx, root);
        this.hexdumpContainer.appendChild(this.hexdump.element);
        window.requestAnimationFrame(() => this.hexdump.regenerateChildren());

        this.reflect = new ReflectView(wasm, ctx, root);
        this.element.appendChild(this.reflect.element);
        this.reflect.onsethighlight = (start, end) => {
            this.highlight = { start, end };
            this.scheduleRefreshMarkings();
        };
        this.reflect.onclearhighlight = () => {
            this.highlight = undefined;
            this.scheduleRefreshMarkings();
        };
        this.reflect.onshowaddr = addr => this.hexdump.scrollToAddr(addr);
        this.reflect.onselect = (item, start, end) => {
            this.selection = { item, start, end };
            this.scheduleRefreshMarkings();
            this.reflect.setSelection(item);
        };
    }

    private scheduleRefreshMarkings() {
        if (!this.refreshMarkingsScheduled) {
            window.requestAnimationFrame(() => {
                this.refreshMarkingsScheduled = false;
                this.refreshMarkings();
            });
            this.refreshMarkingsScheduled = true;
        }
    }

    private refreshMarkings() {
        this.hexdump.clearMarkings();
        if (this.selection !== undefined) {
            this.hexdump.addSelection(this.selection.start, this.selection.end);
        }
        if (this.highlight !== undefined) {
            this.hexdump.addHighlight(this.highlight.start, this.highlight.end);
        }
        this.hexdump.regenerateChildren();
    }

    protected onResize() {
        this.hexdump.regenerateChildren();
    }

    protected onBeforeReattach() {
        this.lastScroll = this.hexdumpContainer.scrollTop;
    }

    protected onAfterReattach() {
        window.requestAnimationFrame(() => {
            this.hexdumpContainer.scrollTop = this.lastScroll;
        });
    }
}
