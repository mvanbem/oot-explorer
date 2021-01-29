import type * as Wasm from '../pkg';

import { ReflectView } from './reflect_view';
import { $t } from './dollar_t';
import { WasmModule } from './wasm';
import { WMWindow } from './window_manager';

export class ExploreView extends WMWindow {
    private readonly hexdumpContainer: HTMLElement;
    private readonly hexdump: Wasm.HexDumpView;
    private readonly reflect: ReflectView;

    private lastScroll: number = 0;

    constructor(wasm: WasmModule, ctx: Wasm.Context) {
        // TODO: Don't hard-code the title.
        super({ title: 'Explore Current Scene', width: 464 });

        this.hexdumpContainer = $t('div', { className: 'explore-view-hexdump' });
        this.element.appendChild(this.hexdumpContainer);
        this.hexdumpContainer.addEventListener('scroll', () => this.hexdump.regenerateChildren());

        this.hexdump = new wasm.HexDumpView(document, ctx);
        this.hexdumpContainer.appendChild(this.hexdump.element);
        window.requestAnimationFrame(() => this.hexdump.regenerateChildren());

        this.reflect = new ReflectView(wasm, ctx);
        this.element.appendChild(this.reflect.element);
        this.reflect.onsethighlight = (start, end) => this.hexdump.setHighlight(start, end);
        this.reflect.onclearhighlight = () => this.hexdump.clearHighlight();
        this.reflect.onshowaddr = addr => this.hexdump.scrollToAddr(addr);
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
