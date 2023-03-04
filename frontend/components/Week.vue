<template>
    <div class="scroll-outer" ref="outer">
        <div class="scroll-inner" ref="inner" @touchstart="onTouchStart" @touchmove="onTouchMove" @wheel="onWheel">
            <div v-for="hour in hours" class="horizontal line" :style="{ 'grid-row': hour + 2 }"></div>
            <DayTimeline
                :events="props.events"
                v-for="day in days.keys()"
                class="vertical line"
                :style="{ 'grid-column': day + 2 }"
            />
            <div v-for="hour in hours" class="hour line" :style="{ 'grid-row': hour + 2 }">
                {{ hour }}
            </div>
            <div class="hour day"></div>
            <div v-for="day in days.keys()" class="day" :style="{ 'grid-column': day + 2 }">
                {{ days[day] }}
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";
import { CalendarEvent } from "@/utils/CalendarEvent";

const props = defineProps<{
    events: CalendarEvent[];
}>();
// TODO: separate events by day

const outer = ref(null as unknown as HTMLDivElement);
const inner = ref(null as unknown as HTMLDivElement);

const weekdays = dayjs.weekdaysMin(true);
const hours = [...Array(24).keys()];
const week = ref(dayjs().startOf("week"));
const days = computed(() => {
    const result: string[] = weekdays;
    return result;
});

let startHeight = 1;
let startScrollY = 0;
let startDistance = 0;
let startPinchY = 0;
let pinching = false;

function onTouchStart(event: TouchEvent) {
    if (event.touches.length !== 2) return;

    const x0 = event.touches[0].clientX;
    const x1 = event.touches[1].clientX;
    const y0 = event.touches[0].clientY;
    const y1 = event.touches[1].clientY;

    const distanceX = Math.abs(x1 - x0);
    const distanceY = Math.abs(y1 - y0);

    pinching = distanceY > distanceX;
    if (!pinching) return;

    startHeight = inner.value!.offsetHeight;
    startScrollY = outer.value!.scrollTop;

    startDistance = distanceY;
    startPinchY = (y0 + y1) / 2;
}

function onTouchMove(event: TouchEvent) {
    if (!pinching || event.touches.length !== 2) return;
    event.preventDefault();

    const y0 = event.touches[0].clientY;
    const y1 = event.touches[1].clientY;

    const distanceY = Math.abs(y1 - y0);
    const pinchY = (y0 + y1) / 2;

    let scale = distanceY / startDistance;
    let newHeight = startHeight * scale;
    // FIXME: also take into account the min-content height (how though?)
    const maxNewHeight = outer.value!.clientHeight;

    if (newHeight < maxNewHeight) {
        scale *= maxNewHeight / newHeight;
        newHeight = maxNewHeight;
    }

    inner.value!.style.height = newHeight + "px";
    outer.value!.scrollTop = scale * (startPinchY + startScrollY) - pinchY;
}

let startMouseY = 0;
let cumulativeDeltaY = 0;

function onWheel(event: WheelEvent) {
    if (!event.altKey) {
        // Since there's no "wheelstart" event, we have to reset manually.
        // One condition to reset is when the alt key is no longer pressed.
        startMouseY = 0;
        return;
    }
    event.preventDefault();

    // That's arbitrary but seems to work well
    const deltaY = -event.deltaY / 100;
    const mouseY = event.clientY;

    // Another condition to reset is the mouse has moved.
    if (mouseY !== startMouseY) {
        startMouseY = mouseY;
        startHeight = inner.value!.offsetHeight;
        startScrollY = outer.value!.scrollTop;
        cumulativeDeltaY = 0;
    }

    cumulativeDeltaY += deltaY;

    let scale = 1 + cumulativeDeltaY;
    let newHeight = startHeight * scale;
    // FIXME: also take into account the min-content height (how though?)
    const maxNewHeight = outer.value!.clientHeight;

    if (newHeight < maxNewHeight) {
        scale *= maxNewHeight / newHeight;
        newHeight = maxNewHeight;

        // Here we also have to reset some things.
        // Otherwise when you scroll past the limit you have to undo that
        // before scrolling in the opposite direction does anythin visible.
        startHeight = inner.value!.offsetHeight;
        startScrollY = outer.value!.scrollTop;
        cumulativeDeltaY = 0;
    }

    inner.value!.style.height = newHeight + "px";
    outer.value!.scrollTop = scale * (startScrollY + startMouseY) - startMouseY;
}
</script>

<style scoped lang="scss">
$border: 1px solid #aaa;
// TODO: remove redundant borders at the right & bottom boundary of the entire week

.scroll-outer {
    overflow: scroll;
}

.scroll-inner {
    display: grid;
    // default height
    height: 120%;
    grid-template-columns: min-content repeat(7, 1fr);
    grid-template-rows: min-content repeat(24, 1fr);
    // slightly decrease font size on mobile
    font-size: clamp(0.8rem, 1vw + 0.5rem, 1rem);
}

.day {
    position: sticky;
    top: 0;
    z-index: 1;
    grid-row: 1;
    padding: 0.4em;
    text-align: center;
    background-color: #fff;
    border-bottom: $border;
}

.hour {
    grid-column: 1;
    padding: 0.2em;
    text-align: right;
}

.line {
    border-right: $border;
    border-bottom: $border;
}

.horizontal {
    grid-column: 2 / -1;
}

.vertical {
    grid-row: 2 / -1;
}
</style>
