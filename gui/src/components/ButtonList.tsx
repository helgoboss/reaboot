import {children, Component, For, JSX, Show} from "solid-js";

type SecondaryButtonListProps = {
    children: JSX.Element,
}

export const ButtonList: Component<SecondaryButtonListProps> = (props) => {
    const cs = children(() => props.children).toArray()
    return (
        <For each={cs}>
            {(item, index) =>
                <div>
                    <Show when={index() > 0}><div>or</div></Show>
                    {item}
                </div>
            }
        </For>
    );
};
