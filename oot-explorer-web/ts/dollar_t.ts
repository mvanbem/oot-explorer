interface DollarTParams {
    className?: string;
    style?: string;
    textContent?: string;
    children?: Array<Node>;
}
interface DollarTInputParams extends DollarTParams {
    type?: string;
}

export function $t(name: 'input', params?: DollarTInputParams): HTMLInputElement;
export function $t<K extends keyof HTMLElementTagNameMap>(name: K, params?: DollarTParams): HTMLElementTagNameMap[K];
export function $t(name: string, params?: DollarTParams): HTMLElement;

export function $t(name: string, params?: DollarTParams | DollarTInputParams): HTMLElement {
    let element = document.createElement(name);
    if (params?.className) {
        element.className = params.className;
    }
    if (params?.style) {
        element.style.cssText = params.style;
    }
    if (params?.textContent) {
        element.textContent = params.textContent;
    }
    if (params?.children) {
        params.children.map(child => element.appendChild(child));
    }
    if (name === 'input' && (<DollarTInputParams>params)?.type) {
        (<HTMLInputElement>element).type = (<DollarTInputParams>params).type!;
    }
    return element;
}

export function $text(text: string): Text {
    return document.createTextNode(text);
}
