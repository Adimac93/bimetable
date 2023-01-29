<template>
    <div class="event-card">
        <div class="relative-time">{{ event.when.startTime?.fromNow() }}</div>
        <h3 class="heading">{{ event.name }}</h3>
        <div class="info">
            <div class="timing" v-if="event.when.startTime && event.when.endTime">
                <div class="timing-date">{{ event.when.day.format("DD MMM YYYY") }}</div>
                <div class="timing-time">{{ event.when.startTime.format("HH:MM") }} - {{ event.when.endTime.format("HH:MM") }}</div>
            </div>
            <div class="notes">
                There are supposed to be notes here. However, they are not in the test data I made.
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import type { CalendarEvent } from '@/utils/CalendarEvent';

const props = defineProps<{
    event: CalendarEvent
}>();

</script>

<style scoped lang="scss">
.event-card {
    border: 1px solid darkgray;
    padding: 12px;
    margin: 4px;
    border-radius: 8px;
}

.heading {
    margin-block: 6px;
    font-weight: bold;
    font-size: large;
}

.info {
    display: flex;
    flex-flow: row nowrap;
    align-items: center;
    gap: 12px;
}
</style>