import { useParams } from 'react-router-dom';
import { useInitializedGuard } from '_hooks';
import Loading from '_components/loading';

import st from './SiteConnectPage.module.scss';

function SiteConnectPage() {
    const { requestID } = useParams();
    const guardLoading = useInitializedGuard(true);
    return (
        <Loading loading={guardLoading}>
            <div className={st.container}>
                <h1>{requestID}</h1>
            </div>
        </Loading>
    );
}

export default SiteConnectPage;
