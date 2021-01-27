/** Provides a global text status bar. */
export const Status = (() => {
    class Status {
        element: HTMLElement;

        constructor() {
            this.element = document.getElementById('status')!;
        }

        show(msg: string) {
            this.element.classList.remove('hidden');
            this.element.textContent = msg;
        }

        hide() {
            this.element.classList.add('hidden');
        }
    }

    return new Status();
})();
