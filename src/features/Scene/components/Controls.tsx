import '../style.css';
import trashcan from '../../../assets/images/icons/trashcan.svg';
import scissors from '../../../assets/images/icons/scissors.svg';
import dragpointer from '../../../assets/images/icons/dragpointer.svg';
import editpointer from '../../../assets/images/icons/editpointer.png';
import { useControls } from '../hooks/useControls';
import ToggleIcon from '../../../components/ToggleIcon';
import { Control } from '../hooks/type';
import { useAppContext } from '../../../context/appContext';
import { SCENE } from '..';
const Controls = () => {
    const { scene } = useAppContext();
    const {
      toggleTrashMode,
      toggleClippingMode,
      toggleDragMode,
      toggleEditingMode,
      showDeleteConfirmation,
    }: Control = useControls();
    return (
        <div className='Controls'>
            {scene.attrs.input.trash &&
            <>
                {scene.attrs.input.safety ?
                <button data-cy='scene-clear' aria-label='clear canvas' className="button" onClick={() => showDeleteConfirmation()}>
                    Delete All
                </button>
                :
                <dialog data-cy='scene-clear-confirmation' className='dialog' aria-label='confirmation to clear canvas' open>
                    Are you sure? Action can't be UNDONE!
                    <button data-cy='scene-clear' aria-label="yes to delete all confirmation message" onClick={() => scene.clear()} className="button">Yes</button>
                </dialog>
                }
            </>
            }
            <ToggleIcon
                name={SCENE.TRASH}
                src={trashcan}
                fn={toggleTrashMode}
                keyShortcuts='Delete'
                title='trash mode'
                cy='scene-trash'
            />
            <ToggleIcon
                name={SCENE.CLIP}
                src={scissors}
                fn={toggleClippingMode}
                keyShortcuts='c'
                title='clipping mode'
                cy='scene-clip'
            />
            <ToggleIcon
                name={SCENE.DRAG}
                src={dragpointer}
                fn={toggleDragMode}
                keyShortcuts='d'
                title='drag mode'
                cy='scene-drag'
            />
            <ToggleIcon
                name={SCENE.EDIT}
                src={editpointer}
                fn={toggleEditingMode}
                keyShortcuts='e'
                title='editing mode'
                cy='scene-edit'
            />
        </div>
    );
}
export default Controls;