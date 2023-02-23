import dayjs from "./dayjs";

export class CalendarEvent {
    name: string;
    startTime?: dayjs.Dayjs;
    endTime?: dayjs.Dayjs;

    constructor(name: string, start: dayjs.Dayjs, end: dayjs.Dayjs) {
        if (!(start && end)) {
            throw new Error("Invalid event (no start and end)");
        }

        this.name = name;
        this.startTime = start;
        this.endTime = end;
    }

    // Returns the day the event starts on.
    get day() {
        const day = (this.startTime ?? this.endTime)?.startOf("day");
        if (!day) {
            throw new Error("Invalid event (no start and end)");
        }
        return day;
    }
};
