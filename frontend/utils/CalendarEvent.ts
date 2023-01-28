import dayjs from "./dayjs";

export interface CalendarEvent {
    name: string,
    when: {
        day: dayjs.Dayjs,
        startTime?: dayjs.Dayjs,
        endTime?: dayjs.Dayjs,
    }
};
