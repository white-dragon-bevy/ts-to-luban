import { RawConfigLoader } from "../../configs-provider/raw-config-loader";


declare const script: {
	configs: Instance
};

const configLoader = new RawConfigLoader(script.configs)

configLoader.initialize()

configLoader.onRawConfigReloaded.Connect((fileName, fullName, isRemoved) => {
	print(`Config reloaded: ${fileName}, isRemoved: ${isRemoved}`)
})

export = {}