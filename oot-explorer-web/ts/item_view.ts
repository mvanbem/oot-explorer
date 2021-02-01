import type * as Wasm from '../pkg';

import { $t, $text } from './dollar_t';
import { WasmModule } from './wasm';

export interface ReflectItemInfo {
    baseAddr: number;
    fieldName?: string;
    typeString: string;
    valueString?: string;
    expandable: boolean;
    vromStart: number;
    vromEnd: number;
}

export type SetHighlightCallback = (start: number, end: number) => void;
export type ClearHighlightCallback = () => void;
export type ShowAddrCallback = (addr: number) => void;
export type SelectCallback = (item: ItemView, start: number, end: number) => void;

export class ItemView {
    private info: ReflectItemInfo;
    private fields: Wasm.ReflectFieldInfo[];
    private header: HTMLElement;
    private indicator?: HTMLElement;
    private contents?: HTMLElement;
    private expanded?: boolean;
    private expandedFields: ItemView[] = [];

    public element: HTMLElement;
    public onsethighlight?: SetHighlightCallback;
    public onclearhighlight?: ClearHighlightCallback;
    public onshowaddr?: ShowAddrCallback;
    public onselect?: SelectCallback;

    constructor(
        private readonly ctx: Wasm.Context,
        private readonly root: Wasm.ReflectRoot,
        reflect: Wasm.ReflectResult,
        private readonly oddNesting: boolean,
    ) {
        this.oddNesting = oddNesting;

        // TODO: Addr, desc parameters? Or some kind of opaque value to a location?
        this.info = reflect.info;
        this.fields = [];
        for (let i = 0; i < reflect.fieldsCount; ++i) {
            this.fields.push(reflect.getField(i));
        }

        // Generate the skeleton of the item element tree.
        let typeElement;
        this.element = $t('div', {
            className: 'tree-item',
            children: [
                this.header = $t('div', {
                    className: 'tree-item-header',
                    children: [
                        $t('span', {
                            className: 'tree-item-addr',
                            textContent: formatAddr(this.info.vromStart),
                        }),
                        $t('span', { textContent: '  ' }),
                        typeElement = $t('span', { textContent: this.info.typeString }),
                    ],
                }),
            ],
        });
        this.element.classList.add(oddNesting ? 'odd' : 'even');
        this.header.addEventListener('click', e => {
            e.stopPropagation();
            this.handleElementClick();
        });

        this.header.addEventListener('mouseenter', () => this.handleHeaderMouseEnter());
        this.header.addEventListener('mouseleave', () => this.handleHeaderMouseLeave());

        // Add a span element for the item's field name.
        if (this.info.fieldName !== undefined) {
            this.header.insertBefore(
                $t('span', { textContent: this.info.fieldName + ': ' }),
                typeElement);
        }

        // Add a span element for the item's value.
        if (this.info.valueString !== undefined) {
            this.header.appendChild($t('span', { textContent: ' = ' + this.info.valueString }));
        }

        // Add elements for expandable content.
        // TODO: Just check if reflection found any fields! We already did the work!
        if (this.fields.length) {
            this.element.appendChild(
                this.indicator = $t('div', { className: 'tree-item-indicator' }));
            this.element.appendChild(
                this.contents = $t('div', { className: 'tree-item-contents' }));

            this.indicator.addEventListener('click', () => this.handleIndicatorClick());
            this.expanded = false;
        }
    }

    handleElementClick() {
        if (this.onshowaddr) {
            this.onshowaddr(this.info.vromStart);
        }
        if (this.onselect) {
            this.onselect(this, this.info.vromStart, this.info.vromEnd);
        }
    }

    handleHeaderMouseEnter() {
        if (this.onsethighlight) {
            this.onsethighlight(this.info.vromStart, this.info.vromEnd);
        }
    }

    handleHeaderMouseLeave() {
        if (this.onclearhighlight) {
            this.onclearhighlight();
        }
    }

    handleIndicatorClick() {
        if (this.expanded) {
            // Collapse.
            while (this.contents!.firstChild) {
                this.contents!.removeChild(this.contents!.firstChild);
            }
            this.expandedFields = [];
            this.expanded = false;
            this.indicator!.classList.remove('expanded');
        } else {
            this.expandImpl();
        }
    }

    expand() {
        if (this.info.expandable && !this.expanded) {
            this.expandImpl();
        }
    }

    private expandImpl() {
        for (let field of this.fields) {
            let fieldView = new ItemView(
                this.ctx, this.root, field.reflect(this.ctx, this.root), !this.oddNesting);
            this.contents!.appendChild(fieldView.element);
            fieldView.onsethighlight = (start, end) => {
                if (this.onsethighlight) {
                    this.onsethighlight(start, end);
                }
            };
            fieldView.onclearhighlight = () => {
                if (this.onclearhighlight) {
                    this.onclearhighlight();
                }
            };
            fieldView.onshowaddr = addr => {
                if (this.onshowaddr) {
                    this.onshowaddr(addr);
                }
            };
            fieldView.onselect = (item, start, end) => {
                if (this.onselect) {
                    this.onselect(item, start, end);
                }
            };
            this.expandedFields.push(fieldView);
        }
        this.expanded = true;
        this.indicator!.classList.add('expanded');
    }

    setSelection(item: ItemView) {
        if (this === item) {
            this.header.classList.add('select');
        } else {
            this.header.classList.remove('select');
        }
        // NOTE: Always recurse into fields because setting the highlight also clears any previous
        // highlight.
        for (let field of this.expandedFields) {
            field.setSelection(item);
        }
    }
}

function formatAddr(addr: number): string {
    return '0x' + addr.toString(16).padStart(8, '0');
}
