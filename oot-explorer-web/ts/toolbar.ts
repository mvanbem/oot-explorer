export const Toolbar = (() => {
    class Toolbar {
        private readonly element: HTMLElement = document.getElementById('toolbar')!;

        hide() {
            this.element.classList.add('hidden');
        }

        show() {
            this.element.classList.remove('hidden');
        }
    }

    return new Toolbar();
})();
