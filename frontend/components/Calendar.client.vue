<template>
    <div>This is a calendar</div>
    <table class="calendar">
        <thead>
            <tr>
                <th colspan="7"><div class="header">
                    <button @click="changeMonth(-1)">&lt;</button>
                    <span>
                        {{ monthStart.format("MMM YYYY") }}
                    </span>
                    <button @click="changeMonth(1)">&gt;</button>
                </div></th>
            </tr>
            <tr>
                <th v-for="weekday in weekdays">{{ weekday }}</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="week in days">
                <template v-for="day in week">
                    <template v-if="'offset' in day">
                        <td v-if="day.offset" :colspan="day.offset"></td>
                    </template>
                    <CalendarCell
                        v-else
                        :day="day.date.date()"
                        :highlight="day.date.isSame(today, 'day')"
                        :events="eventMap.get(getDateString(day.date)) ?? []"
                    />
                </template>
            </tr>
        </tbody>
    </table>
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";
import type { CalendarEvent } from "@/utils/CalendarEvent";

type CalendarSpace = { date: dayjs.Dayjs, events: CalendarEvent[] } | {
    offset: number
};

function getDateString(date: dayjs.Dayjs) {
    return date.format("YYYY-MM-DD");
}

const events = [
    {
        name: "Bibruspotkanie",
        startTime: 1675004400000, // 2023-01-29 15:00:00
        endTime: 1675008000000, // 2023-01-29 16:00:00
    },
    {
        name: "Coś na pewno",
        startTime: 1675072800000, // 2023-01-29 15:00:00
        endTime: 1675101600000, // 2023-01-29 16:00:00
    },
    {
        name: "Podróż w czasie",
        startTime: 1673082000000,
        endTime: 1673082000001,
    }
];

const eventMap = computed(() => {
    const eventMap = new Map<string, CalendarEvent[]>();

    for (const event of events) {
        const newEvent: CalendarEvent = {
            name: event.name,
            when: {
                day: dayjs(event.startTime).startOf("day"),
                startTime: dayjs(event.startTime),
                endTime: dayjs(event.endTime),
            }
        };

        const dateString = getDateString(newEvent.when.day);
        if (eventMap.has(dateString)) {
            eventMap.get(dateString)!.push(newEvent);
        } else {
            eventMap.set(dateString, [newEvent]);
        }
    }
    return eventMap;
});


const weekdays = dayjs.weekdaysMin(true);

const today = dayjs();
const monthStart = ref(dayjs().date(1).startOf("day"));
const daysInMonth = computed(() => monthStart.value.daysInMonth());
const days = computed(() => {
    const days: CalendarSpace[][] = [];
    // if it's not the first day of the week, add an initial week
    if (monthStart.value.weekday() != 0) {
        days.push([]);
    }

    for (let i = 0; i < daysInMonth.value; i++) {
        const day = monthStart.value.add(i, "days");
        const weekDay = day.weekday();
        if (weekDay == 0) {
            days.push([]);
        }
        days[days.length - 1].push({ date: day, events: [] });
    }

    // add empty cells at beginning and end
    const startEmptyCells = monthStart.value.weekday();
    days[0].unshift({ offset: startEmptyCells });

    return days;
});

function changeMonth(offset: number) {
    monthStart.value = monthStart.value.add(offset, "months");
}
</script>

<style scoped lang="scss">
.header {
    display: flex;
    flex-flow: row nowrap;
    justify-content: space-between;
}

.calendar {
    table-layout: fixed;
}

.cell {
    padding: 4px;
}
</style>
