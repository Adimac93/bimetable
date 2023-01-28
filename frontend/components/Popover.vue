<template>
    <Teleport to="body">
        <div class="popover" :class="[anchorClass]">
            <slot/>
        </div>
    </Teleport>
</template>

<script setup lang="ts">
const props = defineProps<{
    sourceRect?: DOMRect
}>();

const bodyRect = useBodyRect();

function lowest(...nums: number[]) {
    let lowestIdx = 0;
    let lowest = Infinity;
    for (let i = 0; i < nums.length; i++) {
        const num = nums[i];
        if (num < lowest) {
            lowest = num;
            lowestIdx = i;
        }
    }
    return lowestIdx;
}

const anchor = computed(() => {
    if (!props.sourceRect) return { anchor: "top", x: 0, y: 0 };

    // calculate the popover's anchor point
    const rect = props.sourceRect;

    const center = { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
    const distances = {
        left: center.x,
        top: center.y,
        right: bodyRect.value.width - center.x,
        bottom: bodyRect.value.height - center.y
    };
    let anchor = "";
    let aX = 0;
    let aY = 0;
    switch (lowest(distances.left, distances.right, distances.top, distances.bottom)) {
        case 0:
            anchor = "right";
            aX = rect.right;
            aY = center.y;
            break;
        case 1:
            anchor = "left";
            aX = rect.left;
            aY = center.y;
            break;
        case 2:
            anchor = "bottom";
            aX = center.x;
            aY = rect.bottom;
            break;
        case 3:
            anchor = "top";
            aX = center.x;
            aY = rect.top;
            break;
    }
    return { type: anchor, x: aX, y: aY };
});

const anchorX = computed(() => anchor.value.x + "px");
const anchorY = computed(() => anchor.value.y + "px");

const MARGIN = 12;
const anchorClass = computed(() => `anchor-${anchor.value.type}`);

</script>

<style scoped lang="scss">
.popover {
    position: absolute;
    left: v-bind('anchorX');
    top: v-bind('anchorY');
    overflow: hidden;

    background-color: white;
    border: 1px solid black;
    border-radius: 8px;
    padding: 8px;
}

.anchor-top {
    transform: translate(-50%, -100%);
}

.anchor-bottom {
    transform: translate(-50%, 0);
}

.anchor-left {
    transform: translate(-100%, -50%);
}

.anchor-right {
    transform: translate(0, -50%);
}
</style>