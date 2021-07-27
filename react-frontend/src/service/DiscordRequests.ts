import Axios from "./AxiosInstance";

export default class DiscordRequests {
    private readonly axios: Axios;

    public constructor() {
        this.axios = Axios.getInstance();
    }

    public async linkClassToGuild(classId: string, snowflake: string) {
        await this.axios.axios.post(`/classes/${classId}/link`, {
            snowflake
        })
    }

    public async linkAccountToDiscord(snowflake: string) {
        await this.axios.axios.post(`/users/me/link`, {
            snowflake
        })
    }

}