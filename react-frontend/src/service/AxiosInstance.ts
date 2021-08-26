import axios, {AxiosInstance} from "axios";

export default class Axios {
    private static INSTANCE: Axios;

    private constructor() {
        this._axios = axios.create({
            baseURL: `${window.location.protocol}//${window.location.host}/api`
        })
    }

    private _axios: AxiosInstance;

    get axios(): AxiosInstance {
        return this._axios;
    }

    public static getInstance(): Axios {
        if (!this.INSTANCE)
            this.INSTANCE = new Axios();
        return this.INSTANCE
    }

    public setAxios(auth?: string) {
        if (auth) {
            this._axios = axios.create({
                baseURL: `${window.location.protocol}//${window.location.host}/api`,
                headers: {
                    Authorization: auth
                }
            })
        } else {
            this._axios = axios.create({
                baseURL: `${window.location.protocol}//${window.location.host}/api`
            })
        }
    }


}
