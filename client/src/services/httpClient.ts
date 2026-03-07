import { Axios, type AxiosRequestConfig, type AxiosResponse } from "axios";

type RequestDataType = Record<string, any>;

export class ApiService<D> extends Axios {
  private config: AxiosRequestConfig<D> = {};

  constructor(config?: AxiosRequestConfig<D>) {
    super();
    Object.assign(this.config, config);
  }

  get<T = any, R = AxiosResponse<T>, D = RequestDataType>(
    url: string,
    config?: AxiosRequestConfig<D>,
  ): Promise<R> {
    const cfg = {
      ...this.config,
      ...config,
    };
    try {
      return super.get(url, cfg);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  post<T = any, R = AxiosResponse<T>, D = RequestDataType>(
      url: string,
      data?: D,
      config?: AxiosRequestConfig<D>
  ): Promise<R> {
    const cfg = {
      ...this.config,
      ...config,
    };

    try {
      return super.post(url, data, cfg);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  put<T = any, R = AxiosResponse<T>, D = any>(url: string, data?: D, config?: AxiosRequestConfig<D>): Promise<R> {
    const cfg = {
      ...this.config,
      ...config,
    };

    try {
      return super.put(url, data, cfg);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  delete<T = any, R = AxiosResponse<T>, D = any>(url: string, config?: AxiosRequestConfig<D>): Promise<R> {
    const cfg = {
      ...this.config,
      ...config,
    };

    try {
      return super.delete(url, cfg);
    } catch (error) {
      return Promise.reject(error);
    }
  }
}

export default new ApiService<RequestDataType>({
  baseURL: "http://localhost:3000",
  headers: {
    "Content-Type": "application/json",
    // "Access-Control-Allow-Origin": "*",
    // Accept: "application/json",
  },
});
