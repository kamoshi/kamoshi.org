import { createSignal } from "solid-js"


export default function Tab(props: any) {
  const [state, setState] = createSignal(false);
  const toggle = () => setState(!state());
    
  return (
    <section class="c-tab" classList={{active: state()}}>
      <button class="c-tab__button" onClick={toggle}>
        <img class="chevron" src="/static/svg/chevron.svg" alt=""/>
        <span class="title">{props.title}</span>
      </button>
      {state() && <div>{props.children}</div>}
    </section>
  )
}
