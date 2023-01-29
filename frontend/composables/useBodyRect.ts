const bodyRect = ref(document.body.getBoundingClientRect());

window.addEventListener("resize", () => {
    bodyRect.value = document.body.getBoundingClientRect();
})

export const useBodyRect = () => {
    return bodyRect;
}
