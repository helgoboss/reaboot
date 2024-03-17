import {MainService} from "./main-service.ts";

export class DummyMainService implements MainService {
    async getMainReaperResourceDir() {
        return Math.random() < 0.5 ? undefined : "/bla/foo";
    }
}