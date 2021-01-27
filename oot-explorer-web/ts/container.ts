/** Displays one element at a time in the `#container` element. */
export const Container = (() => {
    class Container {
        element: HTMLElement;

        constructor() {
            this.element = document.getElementById('container')!;
        }

        setView(view: HTMLElement) {
            while (this.element.lastChild !== null) {
                this.element.removeChild(this.element.lastChild);
            }
            this.element.appendChild(view);
        }

        getBoundingClientRect() {
            return this.element.getBoundingClientRect();
        }
    }

    return new Container();
})();
