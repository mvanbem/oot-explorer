import type * as Wasm from '../pkg';

import { $t } from './dollar_t';
import {
    ClearHighlightCallback, ItemView, SelectCallback, SetHighlightCallback, ShowAddrCallback,
} from './item_view';
import { WasmModule } from './wasm';

export class ReflectView {
    element: HTMLElement;
    rootItem: ItemView;
    onsethighlight?: SetHighlightCallback;
    onclearhighlight?: ClearHighlightCallback;
    onshowaddr?: ShowAddrCallback;
    onselect?: SelectCallback;

    // TODO: addr, desc parameters? Or should roots be retrieve from wasm endpoints?
    constructor(wasm: WasmModule, ctx: Wasm.Context) {
        this.element = $t('div', { className: 'explore-view-tree' });

        this.element.appendChild((this.rootItem =
            new ItemView(ctx, false, wasm.reflect_inside_the_deku_tree_scene(ctx))).element);
        this.rootItem.onsethighlight = (start, end) => {
            if (this.onsethighlight) {
                this.onsethighlight(start, end);
            }
        };
        this.rootItem.onclearhighlight = () => {
            if (this.onclearhighlight) {
                this.onclearhighlight();
            }
        };
        this.rootItem.onshowaddr = addr => {
            if (this.onshowaddr) {
                this.onshowaddr(addr);
            }
        };
        this.rootItem.onselect = (item, start, end) => {
            if (this.onselect) {
                this.onselect(item, start, end);
            }
        };
        this.rootItem.expand();
    }

    setSelection(item: ItemView) {
        this.rootItem.setSelection(item);
    }
}
